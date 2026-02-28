use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::{LockfileData, RiotAuth};
use crate::error::ValorantError;

#[derive(Clone)]
pub struct ValorantClient {
    pub(crate) http: Client,
    pub(crate) lockfile: LockfileData,
    pub(crate) auth: Arc<RwLock<RiotAuth>>,
    pub(crate) client_version: String,
}

impl ValorantClient {
    /// Connect to running Valorant instance
    pub async fn connect() -> Result<Self, ValorantError> {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let lockfile = LockfileData::read()?;
        let auth = RiotAuth::fetch(&http, &lockfile).await?;
        let client_version = fetch_client_version(&http, &lockfile).await
            .unwrap_or_else(|_| "release-10.03.0".to_string());

        Ok(Self {
            http,
            lockfile,
            auth: Arc::new(RwLock::new(auth)),
            client_version,
        })
    }

    pub(crate) fn local_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.lockfile.protocol, self.lockfile.port)
    }

    pub(crate) async fn pd_url(&self) -> String {
        let auth = self.auth.read().await;
        format!("https://pd.{}.a.pvp.net", auth.shard)
    }

    pub(crate) async fn glz_url(&self) -> String {
        let auth = self.auth.read().await;
        format!("https://glz-{}-1.{}.a.pvp.net", auth.region, auth.shard)
    }

    pub(crate) async fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let auth = self.auth.read().await;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization",
            format!("Bearer {}", auth.access_token).parse().unwrap());
        headers.insert("X-Riot-Entitlements-JWT",
            auth.entitlements_token.parse().unwrap());
        headers.insert("X-Riot-ClientVersion",
            self.client_version.parse().unwrap()); 
        headers.insert("X-Riot-ClientPlatform",
            "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9".parse().unwrap());
        headers
    }

    pub async fn puuid(&self) -> String {
        self.auth.read().await.puuid.clone()
    }

    pub async fn get_auth(&self) -> RiotAuth {
        self.auth.read().await.clone()
    }

    /// Generic GET against the PD (Player Data) cluster.
    /// Use this for endpoints the library doesn't yet wrap natively.
    pub async fn raw_get_pd(&self, path: &str) -> Result<serde_json::Value, ValorantError> {
        let url = format!("{}{}", self.pd_url().await, path);
        let resp = self.http.get(&url).headers(self.auth_headers().await).send().await?;
        if !resp.status().is_success() {
            return Err(ValorantError::ApiError {
                status: resp.status().as_u16(),
                message: path.to_string(),
            });
        }
        Ok(resp.json().await?)
    }

    /// Generic GET against the GLZ (Game Lobby Zone) cluster.
    pub async fn raw_get_glz(&self, path: &str) -> Result<serde_json::Value, ValorantError> {
        let url = format!("{}{}", self.glz_url().await, path);
        let resp = self.http.get(&url).headers(self.auth_headers().await).send().await?;
        if !resp.status().is_success() {
            return Err(ValorantError::ApiError {
                status: resp.status().as_u16(),
                message: path.to_string(),
            });
        }
        Ok(resp.json().await?)
    }

    /// Generic PUT against the PD cluster (e.g. name-service).
    pub async fn raw_put_pd<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<serde_json::Value, ValorantError> {
        let url = format!("{}{}", self.pd_url().await, path);
        let resp = self.http
            .put(&url)
            .headers(self.auth_headers().await)
            .json(body)
            .send().await?;
        if !resp.status().is_success() {
            return Err(ValorantError::ApiError {
                status: resp.status().as_u16(),
                message: path.to_string(),
            });
        }
        Ok(resp.json().await?)
    }
}

async fn fetch_client_version(http: &Client, lockfile: &LockfileData) -> Result<String, ValorantError> {
    let base = format!("{}://127.0.0.1:{}", lockfile.protocol, lockfile.port);
    let resp: serde_json::Value = http
        .get(format!("{}/product-session/v1/external-sessions", base))
        .basic_auth("riot", Some(&lockfile.password))
        .send().await?.json().await?;
    
    if let Some(map) = resp.as_object() {
        for (_, session) in map {
            if let Some(version) = session["version"].as_str() {
                if !version.is_empty() {
                    return Ok(version.to_string());
                }
            }
        }
    }
    Err(ValorantError::ApiError { status: 404, message: "version not found".into() })
}
