use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchDetails {
    #[serde(rename = "matchInfo")]
    pub match_info: serde_json::Value,
    pub players: Vec<MatchPlayer>,
    pub teams: Vec<serde_json::Value>,
    #[serde(rename = "roundResults")]
    pub round_results: Vec<serde_json::Value>,
    pub kills: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchPlayer {
    pub subject: String,
    #[serde(rename = "gameName")]
    pub game_name: String,
    #[serde(rename = "tagLine")]
    pub tag_line: String,
    #[serde(rename = "teamId")]
    pub team_id: String,
    #[serde(rename = "characterId")]
    pub character_id: String,
    pub stats: PlayerStats,
    #[serde(rename = "competitiveTier")]
    pub competitive_tier: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub score: u32,
    #[serde(rename = "roundsPlayed")]
    pub rounds_played: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    #[serde(rename = "playtimeMillis")]
    pub playtime_millis: u64,
    #[serde(rename = "abilityCasts")]
    pub ability_casts: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatchHistoryEntry {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    #[serde(rename = "GameStartTime")]
    pub game_start_time: i64,
    #[serde(rename = "QueueID")]
    pub queue_id: String,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}
