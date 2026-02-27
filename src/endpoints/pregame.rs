use crate::client::ValorantClient;
use crate::error::ValorantError;
use crate::models::pregame::{PreGameMatch, PreGamePlayer};

impl ValorantClient {
    pub async fn pregame_player(&self, puuid: &str) -> Result<PreGamePlayer, ValorantError> {
        let url = format!("{}/pregame/v1/players/{}", self.glz_url().await, puuid);
        let resp = self.http.get(&url).headers(self.auth_headers().await)
            .send().await?;
        if resp.status() == 404 { return Err(ValorantError::NotInMatch); }
        Ok(resp.json().await?)
    }

    pub async fn pregame_match(&self, match_id: &str) -> Result<PreGameMatch, ValorantError> {
        let url = format!("{}/pregame/v1/matches/{}", self.glz_url().await, match_id);
        Ok(self.http.get(&url).headers(self.auth_headers().await)
            .send().await?.json().await?)
    }
}
