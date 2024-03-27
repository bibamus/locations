use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::{debug, info};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use serde::{Deserialize, Serialize};
use tokio_postgres::{Config, NoTls, Row};
use uuid::Uuid;

#[derive(Deserialize)]
struct CreatePlace {
    name: String,
    maps_link: String,
}


#[derive(Debug, Serialize, Clone)]
struct Place {
    id: Uuid,
    name: String,
    maps_link: String,
}

type ConnectionPool = Pool<PostgresConnectionManager<MakeTlsConnector>>;
// type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn init_db(pool: &ConnectionPool) {
    info!("Initializing Databsae");
    let conn = pool.get().await.unwrap();
    conn.batch_execute("CREATE TABLE IF NOT EXISTS \
     places \
     (id UUID, \
      name VARCHAR NOT NULL, \
      maps_link VARCHAR NOT NULL)").await.unwrap();
    info!("Database initialised");
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Application starting.");

    let config = get_postgres_config();
    let connector = TlsConnector::builder()
        .build()
        .unwrap();
    let connector = MakeTlsConnector::new(connector);
    let manager = PostgresConnectionManager::new(config, connector);
    // let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    init_db(&pool).await;


    let app = Router::new().route("/place", get(list_places).post(create_place))
        .with_state(pool);


    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_postgres_config() -> Config {
    let host = env::var("POSTGRES_HOST").unwrap();
    let port: u16 = env::var("POSTGRES_PORT").unwrap().parse().unwrap();
    let user = env::var("POSTGRES_USER").unwrap();
    let password = env::var("POSTGRES_PASSWORD").unwrap();
    let database = env::var("POSTGRES_DATABASE").unwrap();

    debug!("Postgres config host={host}, port={port} user={user} password={password} database={database}");

    return Config::new()
        .host(host.as_str())
        .port(port)
        .user(user.as_str())
        .password(password.as_str())
        .dbname(database.as_str()).to_owned();
}

async fn list_places(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let conn = pool.get().await.unwrap();
    let vec = conn.query("SELECT * FROM places", &[]).await.unwrap();
    let places = vec.iter()
        .map(row_to_place)
        .collect::<Vec<_>>();

    return Json(places);
}

fn row_to_place(r: &Row) -> Place {
    let id: Uuid = r.get("id");
    let name: String = r.get("name");
    let maps_link: String = r.get("maps_link");
    return Place {
        id,
        name,
        maps_link,
    };
}

async fn create_place(State(pool): State<ConnectionPool>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let place = Place {
        id: Uuid::new_v4(),
        name: input.name,
        maps_link: input.maps_link,
    };
    let conn = pool.get().await.unwrap();
    conn.execute("INSERT INTO places (id, name, maps_link) VALUES ($1, $2, $3)",
                 &[&place.id, &place.name, &place.maps_link]).await.unwrap();
    return (StatusCode::CREATED, Json(place));
}

type Db = Arc<RwLock<HashMap<Uuid, Place>>>;