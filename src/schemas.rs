use serde_derive::Deserialize;

pub enum InputEvent {
    HarvestEvent(HarvestEvent),
    Unknown { namespace: String, name: String },
}

#[derive(Debug, Deserialize)]
pub enum HarvestEventType {
    #[serde(rename = "DATASET_HARVESTED")]
    DatasetHarvested,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct HarvestEvent {
    #[serde(rename = "type")]
    pub event_type: HarvestEventType,
    #[serde(rename = "fdkId")]
    pub fdk_id: String,
    pub graph: String,
    pub timestamp: i64,
}
