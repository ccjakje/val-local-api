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
}

impl ValorantClient {
    /// Connect to running Valorant instance
    pub async fn connect() -> Result<Self, ValorantError> {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let lockfile = LockfileData::read()?;
        let auth = RiotAuth::fetch(&http, &lockfile).await?;

        Ok(Self {
            http,
            lockfile,
            auth: Arc::new(RwLock::new(auth)),
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
            "release-10.03.0".parse().unwrap()); // Fallback version
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
}
