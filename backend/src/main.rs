use std::collections::HashMap;
use std::env;

use axum::{Json, Router};
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_extra::headers::{Authorization, Header, HeaderMapExt};
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use jsonwebtoken::{Algorithm, decode, decode_header, DecodingKey, TokenData, Validation};
use log::{debug, info};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use serde::{Deserialize, Serialize};
use tokio_postgres::Config;
use uuid::Uuid;
use tower_http::cors::CorsLayer;

use crate::db::{DB, new_db, Place, PlacesDB};

mod db;

#[derive(Deserialize)]
struct CreatePlace {
    name: String,
    maps_link: String,
}


type ConnectionPool = Pool<PostgresConnectionManager<MakeTlsConnector>>;
type DecodingKeys = HashMap<String, DecodingKey>;


async fn init_db(pool: &ConnectionPool) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing Database");
    let conn = pool.get().await?;
    conn.batch_execute("CREATE TABLE IF NOT EXISTS \
     places \
     (id UUID, \
      name VARCHAR NOT NULL, \
      maps_link VARCHAR NOT NULL)").await?;
    info!("Database initialised");
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Application starting.");

    let decoding_keys = load_auth_public_keys().await;

    info!("Loaded keys: {:?}", decoding_keys.len());

    let pg_config = get_postgres_config().unwrap();
    let connector = TlsConnector::builder()
        .build()
        .unwrap();
    let connector = MakeTlsConnector::new(connector);
    let manager = PostgresConnectionManager::new(pg_config, connector);
    let pool = Pool::builder().build(manager).await.unwrap();

    init_db(&pool).await.unwrap();

    let db = new_db(pool);


    let app = Router::new()
        .route("/place", get(list_places).post(create_place))
        .route("/place/:id", get(get_place).patch(update_place).delete(delete_place))
        .route_layer(middleware::from_fn_with_state(decoding_keys, auth))
        .layer(CorsLayer::permissive())
        .with_state(db);


    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


#[derive(Deserialize)]
struct Keys {
    keys: Vec<Key>,
}

#[derive(Deserialize)]
struct Key {
    kid: String,
    #[serde(rename = "use")] use_: String,
    kty: String,
    e: String,
    n: String,
}

async fn load_auth_public_keys() -> HashMap<String, DecodingKey> {
    let url = "https://login.microsoftonline.com/b2748d0a-856e-4184-bda8-831f9ffa8a48/discovery/keys?appid=a4b8584b-9fbd-4bc0-bbfb-363589f1743b";
    let res = reqwest::get(url).await.unwrap();
    let keys: Keys = res.json().await.unwrap();

    let mut map = HashMap::new();

    keys.keys.iter()
        .filter(|key| key.use_ == "sig" && key.kty == "RSA")
        .for_each(|key| {
            debug!("Loaded key: {:?}", key.kid);
            map.insert(key.kid.clone(), DecodingKey::from_rsa_components(&key.n, &key.e).unwrap());
        });

    return map;
}


async fn auth(
    State(decoding_keys): State<DecodingKeys>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header: Option<Authorization<Bearer>> = headers.typed_get();
    let resp = match auth_header {
        None => next.run(request),
        Some(auth_header) => {
            let token = auth_header.token();
            debug!("Token: {:?}", token);
            validate_token(token, decoding_keys);
            next.run(request)
        }
    };
    Ok(resp.await)
}

fn get_postgres_config() -> Result<Config, std::env::VarError> {
    let host = env::var("POSTGRES_HOST")?;
    let port: u16 = env::var("POSTGRES_PORT")?.parse().map_err(|_| env::VarError::NotPresent)?;
    let user = env::var("POSTGRES_USER")?;
    let password = env::var("POSTGRES_PASSWORD")?;
    let database = env::var("POSTGRES_DATABASE")?;

    debug!("Postgres config host={host}, port={port} user={user} password={password} database={database}");

    Ok(Config::new()
        .host(&host)
        .port(port)
        .user(&user)
        .password(&password)
        .dbname(&database)
        .clone())
}

async fn list_places(State(db): State<DB>) -> impl IntoResponse {
    let places = db.list_places().await;
    return Json(places);
}

async fn get_place(State(db): State<DB>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let place = db.get_place(id).await;
    return Json(place);
}


async fn create_place(State(db): State<DB>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let place = Place {
        id: Uuid::new_v4(),
        name: input.name,
        maps_link: input.maps_link,
    };
    let place = db.create_place(place).await;
    return (StatusCode::CREATED, Json(place));
}

async fn update_place(State(db): State<DB>, Path(id): Path<Uuid>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let place = Place {
        id,
        name: input.name.clone(),
        maps_link: input.maps_link.clone(),
    };
    let place = db.update_place(place).await;
    return Json(place);
}


async fn delete_place(State(db): State<DB>, Path(id): Path<Uuid>) -> impl IntoResponse {
    db.delete_place(id).await;
    return StatusCode::NO_CONTENT;
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    upn: String,
    roles: Vec<String>,
}

fn validate_token(token: &str, decoding_keys: DecodingKeys) {
    let header = decode_header(token).unwrap();
    let kid = header.kid.unwrap();
    debug!("Found kid: {:?}", kid);
    match decoding_keys.get(&kid) {
        None => {
            debug!("No key found for kid: {:?}", kid);
        }
        Some(key) => {
            debug!("Found key for kid: {:?}", kid);
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&["api://places.cluster.azure.ludimus.de"]);
            let token_data = decode::<Claims>(token, key, &validation).unwrap();
            let claims = token_data.claims;
            debug!("Claims: {:?}", claims)
        }
    }
}