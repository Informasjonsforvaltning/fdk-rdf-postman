use actix_web::{get, App, HttpServer, Responder};
use futures::{
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};
use fdk_rdf_postman::{
    kafka::run_async_processor,
    metrics::{get_metrics, register_metrics}
};

#[get("/ping")]
async fn ping() -> impl Responder {
    "pong"
}

#[get("/ready")]
async fn ready() -> impl Responder {
    "ok"
}

#[get("/metrics")]
async fn metrics() -> impl Responder {
    match get_metrics() {
        Ok(metrics) => metrics,
        Err(e) => {
            tracing::error!(error = e.to_string(), "unable to gather metrics");
            "".to_string()
        }
    }
}

#[tokio::main]
async fn main() -> () {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_current_span(false)
        .init();

    register_metrics();

    let http_server = tokio::spawn(
        HttpServer::new(|| App::new().service(ping).service(ready).service(metrics))
            .bind(("0.0.0.0", 8080))
            .unwrap_or_else(|e| {
                tracing::error!(error = e.to_string(), "server error");
                std::process::exit(1);
            })
            .run()
            .map(|f| f.map_err(|e| e.into())),
    );

    (0..4)
        .map(|i| tokio::spawn(run_async_processor(i)))
        .chain(std::iter::once(http_server))
        .collect::<FuturesUnordered<_>>()
        .for_each(|result| async {
            result
                .unwrap_or_else(|e| {
                    tracing::error!(error = e.to_string(), "unable to run worker thread");
                    std::process::exit(1);
                })
                .unwrap_or_else(|e| {
                    tracing::error!(error = e.to_string(), "worker failed");
                    std::process::exit(1);
                });
        })
        .await;
}
