use axum::{Json, Router, routing::get};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct CreatePlace {
    name: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/place",
                                  get(|| async { "Hello, World!" })
                                      .post(create_place),
    )
        .with_state("");


    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());


    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_place(Json(input): Json<CreatePlace>) -> String {
    let name = input.name;
    return format!("created {name}");
}