#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = val_local_api::ValorantClient::connect().await?;
    let puuid = client.puuid().await;
    
    match client.coregame_player(&puuid).await {
        Ok(player) => {
            let match_data = client.coregame_match(&player.match_id).await?;
            println!("Map: {}", match_data.map_id);
            println!("Players: {}", match_data.players.len());
        }
        Err(val_local_api::ValorantError::NotInMatch) => {
            println!("Not currently in a match");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
