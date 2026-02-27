use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreGamePlayer {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    #[serde(rename = "Subject")]
    pub puuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreGameMatch {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    #[serde(rename = "MapID")]
    pub map_id: String,
    #[serde(rename = "ModeID")]
    pub mode_id: String,
    #[serde(rename = "Players")]
    pub players: Vec<CoreGameMatchPlayer>,
    #[serde(rename = "Teams")]
    pub teams: Vec<serde_json::Value>,
    #[serde(rename = "RoundResults")]
    pub round_results: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreGameMatchPlayer {
    #[serde(rename = "Subject")]
    pub puuid: String,
    #[serde(rename = "TeamID")]
    pub team_id: String,
    #[serde(rename = "CharacterID")]
    pub character_id: String,
    #[serde(rename = "PlayerIdentity")]
    pub identity: serde_json::Value,
    #[serde(rename = "SeasonalBadgeInfo")]
    pub seasonal_badge: Option<serde_json::Value>,
    #[serde(rename = "IsCoach")]
    pub is_coach: bool,
    #[serde(rename = "IsAssociated")]
    pub is_associated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
