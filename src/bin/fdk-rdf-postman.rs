use actix_web::{get, App, HttpServer, Responder};
use futures::{FutureExt};

#[get("/ping")]
async fn ping() -> impl Responder {
    "pong"
}

#[get("/ready")]
async fn ready() -> impl Responder {
    "ok"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_current_span(false)
        .init();

    HttpServer::new(|| App::new().service(ping).service(ready))
        .bind(("0.0.0.0", 8080))
        .unwrap_or_else(|e| {
            tracing::error!(error = e.to_string(), "server error");
            std::process::exit(1);
        })
        .run()
        .map(|f| f.map_err(|e| e.into()))
        .await
}
