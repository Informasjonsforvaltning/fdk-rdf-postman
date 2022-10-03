use serde_derive::Deserialize;

pub enum InputEvent {
    DatasetEvent(DatasetEvent),
    Unknown { namespace: String, name: String },
}

#[derive(Debug, Deserialize)]
pub enum DatasetEventType {
    #[serde(rename = "DATASET_HARVESTED")]
    DatasetHarvested,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct DatasetEvent {
    #[serde(rename = "type")]
    pub event_type: DatasetEventType,
    #[serde(rename = "fdkId")]
    pub fdk_id: String,
    pub graph: String,
    pub timestamp: i64,
}
