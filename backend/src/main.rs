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
    let pool = Pool::builder().build(manager).await.unwrap();

    init_db(&pool).await;

    let db = new_db(pool);


    let app = Router::new()
        .route("/place", get(list_places).post(create_place))
        .route("/place/:id",get(get_place))
        .with_state(db);


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
