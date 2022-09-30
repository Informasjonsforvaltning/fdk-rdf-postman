use schema_registry_converter::{
    async_impl::schema_registry::{post_schema, SrSettings},
    schema_registry_common::{SchemaType, SuppliedSchema},
};
use serde::Serialize;
use serde_derive::Deserialize;

use crate::error::Error;

pub enum InputEvent {
    DatasetEvent(DatasetEvent),
    Unknown { namespace: String, name: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DatasetEventType {
    #[serde(rename = "DATASET_HARVESTED")]
    DatasetHarvested,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetEvent {
    #[serde(rename = "type")]
    pub event_type: DatasetEventType,
    #[serde(rename = "fdkId")]
    pub fdk_id: String,
    pub graph: String,
    pub timestamp: i64,
}

pub async fn setup_schemas(sr_settings: &SrSettings) -> Result<(), Error> {
    register_schema(
        sr_settings,
        "no.fdk.mqa.DatasetEvent",
        r#"{
            "name": "DatasetEvent",
            "namespace": "no.fdk.mqa",
            "type": "record",
            "fields": [
                {
                    "name": "type",
                    "type": {
                        "type": "enum",
                        "name": "DatasetEventType",
                        "symbols": ["DATASET_HARVESTED"]
                    }
                },
                {"name": "fdkId", "type": "string"},
                {"name": "graph", "type": "string"},
                {"name": "timestamp", "type": "long", "logicalType": "timestamp-millis"}
            ]
        }"#,
    )
        .await?;
    Ok(())
}

pub async fn register_schema(
    sr_settings: &SrSettings,
    name: &str,
    schema_str: &str,
) -> Result<(), Error> {
    tracing::info!(name, "registering schema");

    let schema = post_schema(
        sr_settings,
        name.to_string(),
        SuppliedSchema {
            name: Some(name.to_string()),
            schema_type: SchemaType::Avro,
            schema: schema_str.to_string(),
            references: vec![],
        },
    )
    .await?;

    tracing::info!(id = schema.id, name, "schema succesfully registered");
    Ok(())
}
