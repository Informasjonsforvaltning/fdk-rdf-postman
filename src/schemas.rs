use serde::{Deserialize, Serialize};

pub enum InputEvent {
    HarvestEvent(HarvestEvent),
    Unknown { namespace: String, name: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum HarvestEventType {
    #[serde(rename = "DATASET_HARVESTED")]
    DatasetHarvested,
    #[serde(rename = "DATASET_REASONED")]
    DatasetReasoned,
    #[serde(rename = "DATASET_REMOVED")]
    DatasetRemoved,
    #[serde(rename = "CONCEPT_HARVESTED")]
    ConceptHarvested,
    #[serde(rename = "CONCEPT_REASONED")]
    ConceptReasoned,
    #[serde(rename = "CONCEPT_REMOVED")]
    ConceptRemoved,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvestEvent {
    #[serde(rename = "type")]
    pub event_type: HarvestEventType,
    #[serde(rename = "fdkId")]
    pub fdk_id: String,
    pub graph: String,
    pub timestamp: i64,
}
