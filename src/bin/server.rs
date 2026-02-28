use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, Sse}},
    routing::{get, post},
    Json, Router,
};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::cors::CorsLayer;
use serde_json::json;
use val_local_api::{ValorantClient, ValorantError, LogWatcher, LogEvent};

#[derive(Clone)]
struct AppState {
    client: Arc<ValorantClient>,
    log_tx: broadcast::Sender<LogEvent>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ValorantClient::connect().await.expect("Failed to connect to Valorant");
    
    let (log_watcher, _rx) = LogWatcher::new();
    let log_tx = log_watcher.sender();
    log_watcher.start()?;
    
    let state = AppState {
        client: Arc::new(client),
        log_tx,
    };

    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/auth", get(auth_handler))
        .route("/pregame/match", get(pregame_match_handler))
        .route("/coregame/match", get(coregame_match_handler))
        .route("/coregame/loadouts", get(coregame_loadouts_handler))
        .route("/pd/history", get(history_handler))
        .route("/pd/mmr/{puuid}", get(mmr_handler))
        .route("/pd/match/{match_id}", get(match_detail_handler))
        .route("/pd/names", post(names_handler))
        .route("/pd/lookup/{name}/{tag}", get(lookup_handler))
        .route("/log/events", get(log_events_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9922").await.unwrap();
    println!("Server running on http://127.0.0.1:9922");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn error_response(e: ValorantError) -> axum::response::Response {
    let status = match &e {
        ValorantError::NotInMatch => StatusCode::NOT_FOUND,
        ValorantError::LockfileNotFound => StatusCode::SERVICE_UNAVAILABLE,
        ValorantError::AuthFailed(_) => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({ "error": e.to_string() }))).into_response()
}

async fn status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let puuid = state.client.puuid().await;
    
    if state.client.pregame_player(&puuid).await.is_ok() {
        return Json(json!({ "running": true, "phase": "pregame" }));
    }
    if state.client.coregame_player(&puuid).await.is_ok() {
        return Json(json!({ "running": true, "phase": "ingame" }));
    }    Json(json!({ "running": true, "phase": "menu" }))
}

async fn auth_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let auth = state.client.get_auth().await;
    let puuid = auth.puuid.clone();
    
    let names: Option<Vec<val_local_api::NameEntry>> = state.client.resolve_names(&[puuid.clone()]).await.ok();
    let (name, tag) = names
        .and_then(|mut n| n.pop())
        .map(|e| (e.name, e.tag))
        .unwrap_or_default();

    Json(json!({
        "puuid": puuid,
        "name": name,
        "tag": tag,
        "shard": auth.shard,
        "region": auth.region,
    }))
}

async fn pregame_match_handler(State(state): State<AppState>) -> impl IntoResponse {
    let puuid = state.client.puuid().await;
    match state.client.pregame_player(&puuid).await {
        Ok(player) => match state.client.pregame_match(&player.match_id).await {
            Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
            Err(e) => error_response(e),
        },
        Err(ValorantError::NotInMatch) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not in agent select"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn coregame_match_handler(State(state): State<AppState>) -> impl IntoResponse {
    let puuid = state.client.puuid().await;
    match state.client.coregame_player(&puuid).await {
        Ok(player) => match state.client.coregame_match(&player.match_id).await {
            Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
            Err(e) => error_response(e),
        },
        Err(ValorantError::NotInMatch) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not in match"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn coregame_loadouts_handler(State(state): State<AppState>) -> impl IntoResponse {
    let puuid = state.client.puuid().await;
    match state.client.coregame_player(&puuid).await {
        Ok(player) => match state.client.coregame_loadouts(&player.match_id).await {
            Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
            Err(e) => error_response(e),
        },
        Err(ValorantError::NotInMatch) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not in match"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn history_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let count: u32 = params.get("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);
    let puuid = state.client.puuid().await;
    match state.client.match_history(&puuid, count).await {
        Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn mmr_handler(
    State(state): State<AppState>,
    Path(mut target_puuid): Path<String>,
) -> impl IntoResponse {
    if target_puuid == "me" {
        target_puuid = state.client.puuid().await;
    }
    match state.client.mmr(&target_puuid).await {
        Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn match_detail_handler(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> impl IntoResponse {
    match state.client.match_details(&match_id).await {
        Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn names_handler(
    State(state): State<AppState>,
    Json(puuids): Json<Vec<String>>,
) -> impl IntoResponse {
    match state.client.resolve_names(&puuids).await {
        Ok(data) => (StatusCode::OK, Json(json!(data))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn lookup_handler(
    State(state): State<AppState>,
    Path((name, tag)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.client.lookup_player(&name, &tag).await {
        Ok(puuid) => (StatusCode::OK, Json(json!({ "puuid": puuid }))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn log_events_handler(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.log_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result: Result<LogEvent, tokio_stream::wrappers::errors::BroadcastStreamRecvError>| {
            result.ok().map(|event| {
                let data = match &event {
                    LogEvent::RoundEnded { round_num } => json!({ "type": "round_ended", "round": round_num }),
                    LogEvent::MatchEnded { winning_team } => json!({ "type": "match_ended", "winning_team": winning_team }),
                    LogEvent::PlayerDied => json!({ "type": "player_died" }),
                    LogEvent::BombInteraction { agent } => json!({ "type": "bomb_interaction", "agent": agent }),
                    LogEvent::GameplayStarted => json!({ "type": "gameplay_started" }),
                };
                Ok(Event::default().data(data.to_string()))
            })
        });
    
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
