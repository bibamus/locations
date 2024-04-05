mod db;

use std::env;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::{debug, info};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use serde::{Deserialize};
use tokio_postgres::Config;
use uuid::Uuid;
use crate::db::{DB, new_db, Place, PlacesDB};

#[derive(Deserialize)]
struct CreatePlace {
    name: String,
    maps_link: String,
}




type ConnectionPool = Pool<PostgresConnectionManager<MakeTlsConnector>>;


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
        .route("/place/:id",get(get_place))
        .with_state(db);


    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_postgres_config() -> Result<Config, std::env::VarError> {
    let host = env::var("POSTGRES_HOST")?;
    let port: u16 = env::var("POSTGRES_PORT")?.parse().map_err(|_| std::env::VarError::NotPresent)?;
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

async fn get_place(State(db) : State<DB>, Path(id): Path<Uuid>) -> impl IntoResponse {
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
