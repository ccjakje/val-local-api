# val-local-api

Rust wrapper for Valorant's undocumented local client APIs. Reads auth from the lockfile and exposes all local endpoints — no API key required, no rate limits.

Works as both a **Rust library** (embed in your app) and a **standalone REST server** (use from Python, JavaScript, or any HTTP client).

> **Requires Valorant to be running.**

---

## Features

- Auto-reads lockfile — no manual config
- SSL bypass for Riot's self-signed localhost cert
- Live match data (pregame agent select + in-game)
- Post-match stats — HS%, ADR, ACS, kills/deaths/assists
- MMR, rank history, match history for any player
- Name lookup: `username#tag → PUUID` and reverse
- Real-time log events via SSE stream (round end, bomb plant, player death...)
- Standalone REST server for non-Rust projects

---

## Installation

### As a Rust library

```toml
[dependencies]
val-local-api = { git = "https://github.com/YOUR_USERNAME/val-local-api" }
```

### As a standalone server

```bash
cargo install --git https://github.com/YOUR_USERNAME/val-local-api --features server --bin val-local-api-server
```

Or download a prebuilt binary from [Releases](https://github.com/YOUR_USERNAME/val-local-api/releases).

---

## Quick Start (Rust)

```rust
use val_local_api::ValorantClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ValorantClient::connect().await?;
    let puuid = client.puuid().await;

    match client.coregame_player(&puuid).await {
        Ok(player) => {
            let match_data = client.coregame_match(&player.match_id).await?;
            println!("Map: {}", match_data.map_id);
            println!("Players: {}", match_data.players.len());
        }
        Err(val_local_api::ValorantError::NotInMatch) => {
            println!("Not in a match");
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
```

---

## Quick Start (REST Server)

Start the server (requires Valorant to be running):

```bash
val-local-api-server
# Server running on http://127.0.0.1:9922
```

Then call it from any language:

```python
import requests

# Who am I?
me = requests.get("http://localhost:9922/auth").json()
print(f"{me['name']}#{me['tag']} — {me['region']}")

# Current match
match = requests.get("http://localhost:9922/coregame/match").json()
print(f"Playing on {match['MapID']}")

# Subscribe to round events
import sseclient
events = sseclient.SSEClient("http://localhost:9922/log/events")
for event in events:
    print(event.data)
```

```javascript
// JavaScript / Node.js
const res = await fetch("http://localhost:9922/pd/history?count=10")
const history = await res.json()

// SSE log events
const source = new EventSource("http://localhost:9922/log/events")
source.onmessage = (e) => {
  const event = JSON.parse(e.data)
  if (event.type === "round_ended") console.log(`Round ${event.round} ended`)
}
```

---

## REST API Reference

All endpoints return JSON. Server runs on `http://127.0.0.1:9922`.

### Status & Auth

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/status` | Current game phase |
| `GET` | `/auth` | Own PUUID, name, tag, region |

**`GET /status`**
```json
{
  "running": true,
  "phase": "menu"
}
```
`phase` is one of: `"menu"` `"pregame"` `"ingame"`

**`GET /auth`**
```json
{
  "puuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "name": "houdyxx",
  "tag": "ALTF4",
  "shard": "eu",
  "region": "eu"
}
```

---

### Live Match

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/pregame/match` | Agent select phase data |
| `GET` | `/coregame/match` | Live in-game match data |
| `GET` | `/coregame/loadouts` | Player skins & loadouts |

**`GET /pregame/match`** — returns `404` if not in agent select

**`GET /coregame/match`**
```json
{
  "MatchID": "...",
  "MapID": "/Game/Maps/Ascent/Ascent",
  "ModeID": "...",
  "Players": [
    {
      "Subject": "puuid",
      "TeamID": "Blue",
      "CharacterID": "agent-uuid",
      "IsCoach": false
    }
  ]
}
```

---

### Player Data

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/pd/history?count=20` | Own match history |
| `GET` | `/pd/mmr/{puuid}` | MMR & rank data |
| `GET` | `/pd/match/{match_id}` | Full match details with stats |
| `POST` | `/pd/names` | Resolve PUUIDs → names |
| `GET` | `/pd/lookup/{name}/{tag}` | Name → PUUID |

**`GET /pd/history?count=20`**

`count` — number of matches (default: `20`, max: `100`), any queue

**`GET /pd/mmr/me`**

Use `me` as puuid to get own rank.

**`GET /pd/match/{match_id}`**

Full post-match data including per-round damage, headshots, kills. Use this to calculate HS%, ADR, ACS.

**`POST /pd/names`**

Body: JSON array of PUUIDs
```json
["puuid-1", "puuid-2"]
```
Response:
```json
[
  { "puuid": "puuid-1", "name": "Player1", "tag": "TAG" },
  { "puuid": "puuid-2", "name": "Player2", "tag": "TAG" }
]
```

**`GET /pd/lookup/houdyxx/ALTF4`**
```json
{ "puuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" }
```

---

### Log Events (SSE)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/log/events` | Real-time game events stream |

Server-sent events from `ShooterGame.log`. Connect once, receive events as they happen.

```
GET /log/events
Content-Type: text/event-stream

data: {"type":"gameplay_started"}
data: {"type":"round_ended","round":1}
data: {"type":"bomb_interaction","agent":"Sage"}
data: {"type":"player_died"}
data: {"type":"round_ended","round":2}
data: {"type":"match_ended","winning_team":"Red"}
```

**Event types:**

| Type | Fields | Description |
|------|--------|-------------|
| `gameplay_started` | — | Match begins |
| `round_ended` | `round: number` | Round finished (0-indexed) |
| `match_ended` | `winning_team: "Red"\|"Blue"` | Match over |
| `player_died` | — | Local player died |
| `bomb_interaction` | `agent: string` | Spike plant or defuse |

---

## Error Responses

All errors return JSON with an `error` field:

```json
{ "error": "Valorant lockfile not found — is Valorant running?" }
```

| HTTP Status | Meaning |
|-------------|---------|
| `200` | Success |
| `404` | Not in match / player not found |
| `401` | Auth failed |
| `503` | Valorant not running |
| `500` | Internal error |

---

## Rust API Reference

```rust
// Connect
let client = ValorantClient::connect().await?;

// Identity
client.puuid().await                          // → String
client.get_auth().await                       // → RiotAuth { puuid, shard, region, ... }

// Pregame (agent select)
client.pregame_player(&puuid).await           // → PreGamePlayer  (404 → NotInMatch)
client.pregame_match(&match_id).await         // → PreGameMatch

// Coregame (live match)
client.coregame_player(&puuid).await          // → CoreGamePlayer (404 → NotInMatch)
client.coregame_match(&match_id).await        // → CoreGameMatch
client.coregame_loadouts(&match_id).await     // → serde_json::Value

// Player data
client.match_history(&puuid, count).await     // → Vec<MatchHistoryEntry>
client.match_details(&match_id).await         // → MatchDetails
client.mmr(&puuid).await                      // → MmrData
client.resolve_names(&[puuid]).await          // → Vec<NameEntry>
client.lookup_player(&name, &tag).await       // → String (PUUID)

// Log events
let (watcher, mut rx) = LogWatcher::new();
watcher.start()?;
while let Ok(event) = rx.recv().await {
    match event {
        LogEvent::RoundEnded { round_num } => println!("Round {round_num}"),
        LogEvent::MatchEnded { winning_team } => println!("Winner: {winning_team}"),
        LogEvent::PlayerDied => println!("You died"),
        LogEvent::BombInteraction { agent } => println!("{agent} interacted with spike"),
        LogEvent::GameplayStarted => println!("Match started"),
    }
}
```

---

## How It Works

When Valorant runs, it writes a **lockfile** to:
```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
```

The lockfile contains a port and password for the local REST API. This wrapper reads it, authenticates, and forwards requests to:

- **Local** (`https://127.0.0.1:{port}`) — session, auth tokens
- **GLZ** (`https://glz-{region}-1.{shard}.a.pvp.net`) — live match data
- **PD** (`https://pd.{shard}.a.pvp.net`) — match history, MMR, names

Riot uses a self-signed TLS cert on localhost — this wrapper bypasses cert validation for local requests only.

---

## Limitations

- **Windows only** — lockfile path is Windows-specific
- **Valorant must be running** — no offline mode
- **Tokens expire after ~1 hour** — reconnect if you get auth errors
- **No Riot ToS** — use at your own risk; this uses private APIs

---

## License

MIT
