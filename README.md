# val-local-api

A Rust library and standalone wrapper for Valorant's undocumented local client APIs. Valorant exposes private REST endpoints on localhost when running — this crate automatically reads the auth credentials from the lockfile and provides a clean, easy-to-use interface.

It can be used in two ways:
1. As a **Rust Library (`lib`)** — ideal for Tauri applications (e.g., Vanguard overlays or trackers).
2. As a **Standalone Binary (`bin`)** — runs an HTTP server on `localhost:9922` to be easily consumed by Python, Node.js, or other web developers without dealing with SSL/auth headaches.

---

## Features

- **Automatic Authentication:** Reads `C:\Riot Games\Riot Client\Config\lockfile` (or local AppData log paths) to fetch port and password.
- **SSL Bypass:** Riot Client uses a self-signed certificate locally; this wrapper automatically connects safely.
- **Standardized endpoints:** Easy-to-use methods mapping to `coregame`, `pregame`, `pd` (Player Data), and `nameservice` domains.
- **Game Event Watcher:** Watches `%LOCALAPPDATA%\VALORANT\Saved\Logs\ShooterGame.log` for crucial live triggers (e.g., Round Ended, Player Died, Match Over).

## Table of Contents

- [As a Rust Library](#as-a-rust-library)
- [As a Standalone Server](#as-a-standalone-server)
- [Endpoints Outline](#endpoints-outline)
- [Game Events (Log Watcher)](#game-events-log-watcher)
- [Prerequisites](#prerequisites)

---

## As a Rust Library

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
val-local-api = { git = "https://github.com/ccjakje/val-local-api" }
tokio = { version = "1", features = ["full"] }
```

### Basic Example: Fetch Current Local Match Data

```rust
use val_local_api::ValorantClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Connects automatically reading from lockfile & gets Riot auth tokens
    let client = ValorantClient::connect().await?;
    let my_puuid = client.puuid().await;
    
    // 2. See if the player is currently in a live Match
    match client.coregame_player(&my_puuid).await {
        Ok(player) => {
            let match_data = client.coregame_match(&player.match_id).await?;
            println!("Currently playing on map: {}", match_data.map_id);
            println!("Total players in match: {}", match_data.players.len());
        }
        Err(val_local_api::ValorantError::NotInMatch) => {
            println!("Player is not currently in a match.");
        }
        Err(e) => eprintln!("An error occurred: {}", e),
    }
    
    Ok(())
}
```

---

## As a Standalone Server

If you prefer building your overlay with Python or pure JS without Rust binding glue, you can run the crate as a background REST Server!

### Starting the server
Make sure you have Rust installed and Valorant running, then execute:

```bash
cargo run --features server --bin val-local-api-server
```

The server will start on `http://127.0.0.1:9922` with **CORS enabled** for local frontend cross-origin requests.

### Server API Example (JSON Responses):

- `GET http://127.0.0.1:9922/status` → Checks if the bridge is active and running.
- `GET http://127.0.0.1:9922/auth` → Returns your `puuid`, `shard`, and `region`.

*(Note: Additional endpoint routes can be mapped in `src/bin/server.rs` according to the wrapper methods.)*

---

## Endpoints Outline

The `ValorantClient` exposes Several main module scopes. Typical endpoints available:

### Sessions & Identity
- `client.puuid()` - Extract Player UUID
- `client.get_auth()` - Returns region, shard, auth tokens.
- `client.session()` - Get the current client state and phase.
- `client.client_version()` - Valorant release string.

### Core-Game (Live Match)
- `client.coregame_player("puuid")` - Find out if player is in a match and their MatchID.
- `client.coregame_match("match_id")` - Fetch full match live rosters and map.
- `client.coregame_loadouts("match_id")` - Fetch weapon/skin variants used by players.

### Pre-Game (Agent Select)
- `client.pregame_player("puuid")` - Find match ID of the current agent select queue.
- `client.pregame_match("match_id")` - Get selection states of the lobby.

### Player Data (PD)
- `client.lookup_player("Name", "Tag")` - Resolves riot ID to a PUUID.
- `client.resolve_names(&["puuid1",...])` - Batch resolves PUUIDs to their current Riot IDs.
- `client.match_history("puuid", 10)` - Get competitive game history.
- `client.match_details("match_id")` - Post-match stats and telemetry.
- `client.mmr("puuid")` - Real-time rank and MMR updates.

---

## Game Events (Log Watcher)

Valorant updates local filesystem logs whenever core events occur mid-match. This API parses them and broadcasts strongly-typed events.

```rust
use val_local_api::{LogWatcher, LogEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (watcher, mut rx) = LogWatcher::new();
    watcher.start()?; // Spawns log tail background task

    while let Ok(event) = rx.recv().await {
        match event {
            LogEvent::RoundEnded { round_num } => println!("Round {} abruptly ended!", round_num),
            LogEvent::MatchEnded { winning_team } => println!("Game Over! Team {} won", winning_team),
            LogEvent::PlayerDied => println!("Local player was eliminated!"),
            LogEvent::BombInteraction { agent } => println!("Spike interaction from {}", agent),
            LogEvent::GameplayStarted => println!("Spawn barriers dropped!"),
        }
    }
    
    Ok(())
}
```

---

## Prerequisites
- **OS:** Windows Only (for automatic lockfile/log directory traversal).
- **Game State:** Valorant (or at least Riot Client) MUST be running for the local REST Server or Library to locate the lockfile. 

---

### License
MIT
