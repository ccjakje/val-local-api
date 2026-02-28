// use notify::{Watcher, RecursiveMode, Event};
use tokio::sync::broadcast;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum LogEvent {
    RoundEnded { round_num: u32 },
    MatchEnded { winning_team: String },
    PlayerDied,
    BombInteraction { agent: String },
    GameplayStarted,
}

pub struct LogWatcher {
    sender: broadcast::Sender<LogEvent>,
}

impl LogWatcher {
    pub fn new() -> (Self, broadcast::Receiver<LogEvent>) {
        let (tx, rx) = broadcast::channel(64);
        (Self { sender: tx }, rx)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogEvent> {
        self.sender.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<LogEvent> {
        self.sender.clone()
    }

    pub fn start(&self) -> Result<(), anyhow::Error> {
        let log_path = Self::log_path()?;
        let sender = self.sender.clone();

        tokio::spawn(async move {
            Self::tail_file(log_path, sender).await;
        });

        Ok(())
    }

    fn log_path() -> Result<PathBuf, anyhow::Error> {
        let path = dirs::data_local_dir()
            .ok_or(anyhow::anyhow!("No data dir"))?
            .join("VALORANT/Saved/Logs/ShooterGame.log");
        Ok(path)
    }

    async fn tail_file(path: PathBuf, sender: broadcast::Sender<LogEvent>) {
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::fs::File;

        let file = match File::open(&path).await {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut reader = BufReader::new(file);
        use tokio::io::AsyncSeekExt;
        let _ = reader.seek(std::io::SeekFrom::End(0)).await;

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    if let Some(event) = Self::parse_line(&line) {
                        let _ = sender.send(event);
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn parse_line(line: &str) -> Option<LogEvent> {
        if line.contains("AShooterGameState::OnRoundEnded for round") {
            let round = line.split("round '").nth(1)?
                .split('\'').next()?
                .parse::<u32>().ok()?;
            return Some(LogEvent::RoundEnded { round_num: round });
        }
        if line.contains("Match Ended: Completion State") {
            let team = if line.contains("Winning Team:") {
                line.split("Winning Team: '").nth(1)?
                    .split('\'').next()?.to_string()
            } else { "unknown".to_string() };
            return Some(LogEvent::MatchEnded { winning_team: team });
        }
        if line.contains("_PostDeath_PC") && line.contains("AcknowledgePawn") && !line.contains("PrevPawn") {
            if line.contains("ClientRestart_Implementation") {
                return Some(LogEvent::PlayerDied);
            }
        }
        if line.contains("BombInteractionBuff_C") {
            let agent = line.split("InternalOnActiveGameplayEffectAdded ").nth(1)?
                .split('_').next()?.to_string();
            return Some(LogEvent::BombInteraction { agent });
        }
        if line.contains("Gameplay started at local time 0.") {
            return Some(LogEvent::GameplayStarted);
        }
        None
    }
}
