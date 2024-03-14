use axum::{Json, Router, routing::get};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct CreateLocation {
    name: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/location",
                                  get(|| async { "Hello, World!" })
                                      .post(create_location),
    )
        .with_state("");


    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());


    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_location(Json(input): Json<CreateLocation>) {
    dbg!(input.name);
}