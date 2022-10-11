use std::env;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::Serialize;
use crate::{
    error::Error,
    schemas::{HarvestEvent, HarvestEventType},
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

#[derive(Debug, Serialize)]
struct DiffStoreID {
    pub id: String,
}

pub async fn update_diff_store(
    event: HarvestEvent,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    match event.event_type {
        HarvestEventType::DatasetHarvested => {
            Ok(())
        }
        HarvestEventType::DatasetReasoned => {
            post_event_graph_to_diff_store(event, http_client).await
        }
        HarvestEventType::DatasetRemoved => {
            delete_graph_in_diff_store(event, http_client).await
        }
        HarvestEventType::Unknown => {
            Ok(())
        }
    }
}

async fn delete_graph_in_diff_store(
    event: HarvestEvent,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    let response = http_client
        .delete(format!(
            "{}/api/graphs",
            DIFF_STORE_URL.clone()
        ))
        .header("X-API-KEY", DIFF_STORE_KEY.clone())
        .json(&DiffStoreID {id: event.fdk_id.clone()})
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!(
            "Invalid response when deleting {} from diff store: {} - {}",
            event.fdk_id,
            response.status(),
            response.text().await?
        ).into())
    }
}

async fn post_event_graph_to_diff_store(
    event: HarvestEvent,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    let response = http_client
        .post(format!(
            "{}/api/graphs",
            DIFF_STORE_URL.clone()
        ))
        .header("X-API-KEY", DIFF_STORE_KEY.clone())
        .json(&DiffStoreGraph {id: event.fdk_id.clone(), graph: event.graph})
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!(
            "Invalid response from diff store for {}: {} - {}",
            event.fdk_id,
            response.status(),
            response.text().await?
        ).into())
    }
}
