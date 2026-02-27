use crate::client::ValorantClient;
use crate::error::ValorantError;
use crate::models::session::SessionData;

impl ValorantClient {
    /// Get current game session state
    pub async fn session(&self) -> Result<SessionData, ValorantError> {
        let resp: serde_json::Value = self.http
            .get(format!("{}/product-session/v1/external-sessions", self.local_url()))
            .basic_auth("riot", Some(&self.lockfile.password))
            .send().await?.json().await?;
        
        Ok(serde_json::from_value(resp)?)
    }

    /// Get client version (needed for headers)
    pub async fn client_version(&self) -> Result<String, ValorantError> {
        let resp: serde_json::Value = self.http
            .get(format!("{}/product-session/v1/external-sessions", self.local_url()))
            .basic_auth("riot", Some(&self.lockfile.password))
            .send().await?.json().await?;
        
        resp.get("clientVersion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ValorantError::ApiError { 
                status: 404, 
                message: "Client version not found".into() 
            })
    }
}
