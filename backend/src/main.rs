use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct CreatePlace {
    name: String,
}


#[derive(Debug, Serialize, Clone)]
struct Place {
    id: Uuid,
    name: String,
}

#[tokio::main]
async fn main() {
    let db = Db::default();

    let app = Router::new().route("/place", get(list_places).post(create_place))
        .with_state(db);


    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_places(State(db): State<Db>) -> impl IntoResponse {
    let places = db.read().unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>();

    return Json(places);
}

async fn create_place(State(db): State<Db>, Json(input): Json<CreatePlace>) -> impl IntoResponse {
    let place = Place {
        id: Uuid::new_v4(),
        name: input.name,
    };
    db.write().unwrap().insert(place.id, place.clone());
    return (StatusCode::CREATED, Json(place));
}

type Db = Arc<RwLock<HashMap<Uuid, Place>>>;