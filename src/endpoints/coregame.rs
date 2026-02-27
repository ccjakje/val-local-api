use crate::client::ValorantClient;
use crate::error::ValorantError;
use crate::models::session::{CoreGameMatch, CoreGamePlayer};

impl ValorantClient {
    /// Get current match ID for a player
    pub async fn coregame_player(&self, puuid: &str) -> Result<CoreGamePlayer, ValorantError> {
        let url = format!("{}/core-game/v1/players/{}", self.glz_url().await, puuid);
        let resp = self.http.get(&url).headers(self.auth_headers().await)
            .send().await?;
        if resp.status() == 404 { return Err(ValorantError::NotInMatch); }
        Ok(resp.json().await?)
    }

    /// Get full live match data
    pub async fn coregame_match(&self, match_id: &str) -> Result<CoreGameMatch, ValorantError> {
        let url = format!("{}/core-game/v1/matches/{}", self.glz_url().await, match_id);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }

    /// Get player loadouts in current match
    pub async fn coregame_loadouts(&self, match_id: &str) -> Result<serde_json::Value, ValorantError> {
        let url = format!("{}/core-game/v1/matches/{}/loadouts", self.glz_url().await, match_id);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }
}
