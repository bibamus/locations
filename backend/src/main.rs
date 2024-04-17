use std::env;

use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware::{self};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use log::{debug, info};
use serde::Deserialize;
use tokio_postgres::Config;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::auth::{auth, load_auth_public_keys};
use crate::db::{DB, Place, PlacesDB};

mod db;
mod auth;


#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Application starting.");

    let decoding_keys = load_auth_public_keys().await;
    info!("Loaded keys: {:?}", decoding_keys.len());

    let pg_config = get_postgres_config().unwrap();

    let db = DB::new_db(pg_config).await;
    db.init_db().await.unwrap();


    let app = Router::new()
        .route("/place", get(list_places).post(create_place))
        .route("/place/:id", get(get_place).put(update_place).delete(delete_place))
        .route("/place/:id/rating", post(rate_place))
        .route_layer(middleware::from_fn_with_state(decoding_keys, auth))
        .layer(CorsLayer::permissive())
        .with_state(db);


    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


fn get_postgres_config() -> Result<Config, env::VarError> {
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


#[derive(Deserialize)]
struct CreatePlace {
    name: String,
    maps_link: String,
}

async fn list_places(State(db): State<DB>,
                     Extension(claims): Extension<auth::Claims>,
) -> impl IntoResponse {
    let places = db.list_places_with_rating(claims.upn).await;
    return Json(places);
}

async fn get_place(State(db): State<DB>,
                   Extension(claims): Extension<auth::Claims>,
                   Path(id): Path<Uuid>) -> impl IntoResponse {
    let place = db.get_place_with_rating(id, claims.upn).await;
    return Json(place);
}


async fn create_place(State(db): State<DB>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let place = Place::new(input.name, input.maps_link);
    let place = db.create_place(place).await;
    return (StatusCode::CREATED, Json(place));
}

async fn update_place(State(db): State<DB>, Path(id): Path<Uuid>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let mut place = db.get_place(id).await;
    place.name = input.name;
    place.maps_link = input.maps_link;
    let place = db.update_place(place).await;
    return Json(place);
}


async fn delete_place(State(db): State<DB>, Path(id): Path<Uuid>) -> impl IntoResponse {
    db.delete_place(id).await;
    return StatusCode::NO_CONTENT;
}

async fn rate_place(State(db): State<DB>,
                    Path(id): Path<Uuid>,
                    Extension(claims): Extension<auth::Claims>,
                    Json(input): Json<i32>) -> impl IntoResponse {
    let place = db.rate_place(id, claims.upn, input).await;
    return Json(place);
}
