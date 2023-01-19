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
    pub static ref POSTMAN_TYPE: PostmanType = get_postman_type(env::var("POSTMAN_TYPE").unwrap_or("dataset".to_string()));
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

#[derive(Clone)]
pub enum PostmanType {
    Dataset,
    Concept,
    Unknown,
}

pub enum DiffStoreAction {
    PostGraph,
    DeleteGraph,
    Nothing,
}

pub async fn update_diff_store(
    event: HarvestEvent,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    match event_to_action(event.event_type) {
        DiffStoreAction::PostGraph => {
            post_event_graph_to_diff_store(event, http_client).await
        }
        DiffStoreAction::DeleteGraph => {
            delete_graph_in_diff_store(event, http_client).await
        }
        DiffStoreAction::Nothing => {
            Ok(())
        }
    }
}

fn get_postman_type(postman_type_string: String) -> PostmanType {
    match postman_type_string.as_str() {
        "dataset" => {
            PostmanType::Dataset
        }
        "concept" => {
            PostmanType::Concept
        }
        _ => {
            tracing::error!(postman_type_string, "unknown postman type");
            PostmanType::Unknown
        }
    }
}

fn event_to_action(event_type: HarvestEventType) -> DiffStoreAction {
    match (event_type, POSTMAN_TYPE.clone()) {
        (HarvestEventType::DatasetReasoned, PostmanType::Dataset) => {
            DiffStoreAction::PostGraph
        }
        (HarvestEventType::DatasetRemoved, PostmanType::Dataset) => {
            DiffStoreAction::DeleteGraph
        }
        (HarvestEventType::ConceptReasoned, PostmanType::Concept) => {
            DiffStoreAction::PostGraph
        }
        (HarvestEventType::ConceptRemoved, PostmanType::Concept) => {
            DiffStoreAction::DeleteGraph
        }
        _ => {
            DiffStoreAction::Nothing
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
            DIFF_STORE_URL.clone().as_str()
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
            event.fdk_id.as_str(),
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
            DIFF_STORE_URL.clone().as_str()
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
            event.fdk_id.as_str(),
            response.status(),
            response.text().await?
        ).into())
    }
}
