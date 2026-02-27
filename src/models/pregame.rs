use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreGamePlayer {
    #[serde(rename = "Subject")]
    pub puuid: String,
    #[serde(rename = "MatchID")]
    pub match_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreGameMatch {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}
