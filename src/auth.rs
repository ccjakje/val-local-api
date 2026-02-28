use crate::error::ValorantError;
use std::path::PathBuf;
use base64::Engine;

#[derive(Debug, Clone)]
pub struct LockfileData {
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

impl LockfileData {
    pub fn read() -> Result<Self, ValorantError> {
        let path = lockfile_path()?;
        let content = std::fs::read_to_string(&path)
            .map_err(|_| ValorantError::LockfileNotFound)?;
        
        let parts: Vec<&str> = content.trim().split(':').collect();
        if parts.len() < 5 {
            return Err(ValorantError::LockfileMalformed);
        }
        
        Ok(LockfileData {
            port: parts[2].parse().map_err(|_| ValorantError::LockfileMalformed)?,
            password: parts[3].to_string(),
            protocol: parts[4].to_string(),
        })
    }
}

fn lockfile_path() -> Result<PathBuf, ValorantError> {
    let candidates = vec![
        PathBuf::from(r"C:\Riot Games\Riot Client\Config\lockfile"),
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("Riot Games/Riot Client/Config/lockfile"),
    ];
    
    candidates.into_iter()
        .find(|p| p.exists())
        .ok_or(ValorantError::LockfileNotFound)
}

#[derive(Debug, Clone)]
pub struct RiotAuth {
    pub access_token: String,
    pub entitlements_token: String,
    pub puuid: String,
    pub shard: String,
    pub region: String,
}

impl RiotAuth {
    pub async fn fetch(client: &reqwest::Client, lockfile: &LockfileData) -> Result<Self, ValorantError> {
        let base = format!("{}://127.0.0.1:{}", lockfile.protocol, lockfile.port);
        
        let resp: serde_json::Value = client
            .get(format!("{}/entitlements/v1/token", base))
            .basic_auth("riot", Some(&lockfile.password))
            .send().await?
            .json().await?;

        let access_token = resp["accessToken"].as_str()
            .ok_or_else(|| ValorantError::AuthFailed("missing accessToken".into()))?.to_string();
        let entitlements_token = resp["token"].as_str()
            .ok_or_else(|| ValorantError::AuthFailed("missing token".into()))?.to_string();
        let puuid = resp["subject"].as_str()
            .ok_or_else(|| ValorantError::AuthFailed("missing subject".into()))?.to_string();

        let (shard, region) = parse_region_from_token(&access_token)?;

        Ok(RiotAuth { access_token, entitlements_token, puuid, shard, region })
    }
}

fn parse_region_from_token(token: &str) -> Result<(String, String), ValorantError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err(ValorantError::AuthFailed("invalid JWT".into()));
    }

    // Pad base64 if needed
    let padded = {
        let s = parts[1];
        let padding = (4 - s.len() % 4) % 4;
        format!("{}{}", s, "=".repeat(padding))
    };

    let payload = base64::engine::general_purpose::URL_SAFE
        .decode(&padded)
        .map_err(|_| ValorantError::AuthFailed("JWT decode failed".into()))?;

    let json: serde_json::Value = serde_json::from_slice(&payload)?;

    // Try multiple known fields where region appears
    let shard = json.pointer("/acct/country")
        .or_else(|| json.get("region"))
        .or_else(|| json.get("shard"))
        .and_then(|v| v.as_str())
        .map(map_region_to_shard)
        .unwrap_or_else(|| "eu".to_string());

    // region and shard are often the same for Valorant
    Ok((shard.clone(), shard))
}

fn map_region_to_shard(region: &str) -> String {
    match region.to_lowercase().as_str() {
        "euw" | "eune" | "eu" | "tr" | "ru" => "eu",
        "na" | "us" | "br" | "latam" | "lan" | "las" => "na",
        "ap" | "kr" | "jp" | "oce" | "sea" => "ap",
        other => other,
    }
    .to_string()
}
