use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NameEntry {
    pub puuid: String,
    pub name: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MmrData {
    #[serde(rename = "Subject")]
    pub puuid: String,
    #[serde(rename = "LatestCompetitiveUpdate")]
    pub latest_update: Option<serde_json::Value>,
    #[serde(rename = "QueueSkills")]
    pub queue_skills: serde_json::Value,
}
