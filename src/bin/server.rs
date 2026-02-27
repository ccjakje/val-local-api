use axum::{
    routing::get,
    Router, Json,
};
use tower_http::cors::CorsLayer;
use val_local_api::ValorantClient;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/auth", get(auth_handler))
        // TODO: add all the other endpoints mapped to the client methods
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9922").await.unwrap();
    println!("Server running on http://127.0.0.1:9922");
    axum::serve(listener, app).await.unwrap();
}

async fn status_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"running": true, "phase": "unknown"}))
}

async fn auth_handler() -> Json<serde_json::Value> {
    match ValorantClient::connect().await {
        Ok(client) => {
            // Need to spawn a task or just lock it briefly
            let auth = client.get_auth().await;
            Json(serde_json::json!({
                "puuid": auth.puuid,
                "shard": auth.shard,
                "region": auth.region,
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
