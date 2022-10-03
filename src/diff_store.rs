use std::env;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::Serialize;
use crate::{
    error::Error,
    schemas::HarvestEvent,
};

lazy_static! {
    pub static ref DIFF_STORE_URL: String = env::var("DIFF_STORE_URL").unwrap_or("http://localhost:8090".to_string());
    pub static ref DIFF_STORE_KEY: String = env::var("DIFF_STORE_KEY").unwrap_or("test-key".to_string());
}

#[derive(Debug, Serialize)]
struct DiffStoreGraph {
    pub id: String,
    pub graph: String,
}

pub async fn post_event_graph_to_diff_store(
    event: HarvestEvent,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    let response = http_client
        .post(format!(
            "{}/api/graphs",
            DIFF_STORE_URL.clone()
        ))
        .header("X-API-KEY", DIFF_STORE_KEY.clone())
        .json(&DiffStoreGraph {id: event.fdk_id, graph: event.graph})
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!(
            "Invalid response from diff store: {} - {}",
            response.status(),
            response.text().await?
        )
            .into())
    }
}
