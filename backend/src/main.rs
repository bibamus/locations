use axum::{Json, Router, routing::get};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateLocation {
    name: String
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/location",
                                  get(|| async { "Hello, World!" })
                                      .post(create_location),
    )
        .with_state("");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_location(Json(input): Json<CreateLocation>) {
    dbg!(input.name);
}