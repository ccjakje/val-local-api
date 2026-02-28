use crate::client::ValorantClient;
use crate::error::ValorantError;
use crate::models::match_data::{MatchDetails, MatchHistoryEntry};
use crate::models::player::{NameEntry, MmrData};

impl ValorantClient {
    /// Resolve name+tag → PUUID using the name-service.
    /// Passes "Name#Tag" string to the same PUT endpoint used for PUUID→name.
    pub async fn lookup_player(&self, name: &str, tag: &str) -> Result<String, ValorantError> {
        let query = format!("{}#{}", name, tag);
        let url = format!("{}/name-service/v2/players", self.pd_url().await);
        let resp: Vec<serde_json::Value> = self.http
            .put(&url)
            .headers(self.auth_headers().await)
            .json(&vec![&query])
            .send().await?.json().await?;

        resp.into_iter()
            .find(|v| {
                let game_name = v["GameName"].as_str().unwrap_or("").to_lowercase();
                let tag_line  = v["TagLine"].as_str().unwrap_or("").to_lowercase();
                game_name == name.to_lowercase() && tag_line == tag.to_lowercase()
            })
            .and_then(|v| v["Subject"].as_str().map(|s| s.to_string()))
            .ok_or(ValorantError::ApiError { status: 404, message: "Player not found".into() })
    }

    /// PUUID → name + tag for multiple players
    pub async fn resolve_names(&self, puuids: &[String]) -> Result<Vec<NameEntry>, ValorantError> {
        let url = format!("{}/name-service/v2/players", self.pd_url().await);
        let resp: Vec<serde_json::Value> = self.http
            .put(&url)
            .headers(self.auth_headers().await)
            .json(puuids)
            .send().await?.json().await?;
            
        Ok(resp.iter().map(|v| NameEntry {
            puuid: v["Subject"].as_str().unwrap_or("").to_string(),
            name: v["GameName"].as_str().unwrap_or("").to_string(),
            tag: v["TagLine"].as_str().unwrap_or("").to_string(),
        }).collect())
    }

    /// Get match history for a player
    pub async fn match_history(&self, puuid: &str, count: u32) -> Result<Vec<MatchHistoryEntry>, ValorantError> {
        let url = format!("{}/match-history/v1/history/{}?startIndex=0&endIndex={}&queue=competitive", 
            self.pd_url().await, puuid, count);
        let resp: serde_json::Value = self.http.get(&url)
            .headers(self.auth_headers().await)
            .send().await?.json().await?;
            
        if let Some(history) = resp.get("History").and_then(|h| h.as_array()) {
            let entries: Result<Vec<MatchHistoryEntry>, _> = history.iter()
                .map(|v| serde_json::from_value(v.clone()))
                .collect();
            Ok(entries?)
        } else {
            Ok(vec![])
        }
    }

    /// Get full match details (post-match stats, HS%, damage, etc.)
    pub async fn match_details(&self, match_id: &str) -> Result<MatchDetails, ValorantError> {
        let url = format!("{}/match-details/v1/matches/{}", self.pd_url().await, match_id);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }

    /// Get MMR / rank data for a player
    pub async fn mmr(&self, puuid: &str) -> Result<MmrData, ValorantError> {
        let url = format!("{}/mmr/v1/players/{}", self.pd_url().await, puuid);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }

    /// Get competitive leaderboard for a region
    pub async fn leaderboard(&self, season_id: &str, start: u32, size: u32) -> Result<serde_json::Value, ValorantError> {
        let url = format!("{}/mmr/v1/leaderboards/affinity/{}/queue/competitive/season/{}?startIndex={}&size={}", 
            self.pd_url().await, 
            self.auth.read().await.region,
            season_id, start, size);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }
}
