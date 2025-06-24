use reqwest::Client;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};
use base64::{Engine as _, engine::general_purpose};
use std::env;
use tauri::Emitter;

#[derive(Debug)]
pub struct LeagueError {
    message: String,
}

impl std::fmt::Display for LeagueError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LeagueError {}

impl From<&str> for LeagueError {
    fn from(msg: &str) -> Self {
        LeagueError { message: msg.to_string() }
    }
}

impl From<String> for LeagueError {
    fn from(msg: String) -> Self {
        LeagueError { message: msg }
    }
}

impl From<reqwest::Error> for LeagueError {
    fn from(err: reqwest::Error) -> Self {
        LeagueError { message: err.to_string() }
    }
}

impl From<std::io::Error> for LeagueError {
    fn from(err: std::io::Error) -> Self {
        LeagueError { message: err.to_string() }
    }
}

impl From<std::num::ParseIntError> for LeagueError {
    fn from(err: std::num::ParseIntError) -> Self {
        LeagueError { message: err.to_string() }
    }
}

pub struct LeagueClient {
    client: Client,
    base_url: String,
    auth_header: String,
    port: u16,
}

impl LeagueClient {
    pub async fn new() -> Result<Self, LeagueError> {
        let lockfile_path = Self::find_lockfile()?;
        let lockfile_content = fs::read_to_string(lockfile_path)?;
        
        let parts: Vec<&str> = lockfile_content.split(':').collect();
        if parts.len() < 5 {
            return Err("Invalid lockfile format".into());
        }
        
        let port: u16 = parts[2].parse()?;
        let password = parts[3];
        
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        
        let auth = format!("riot:{}", password);
        let auth_encoded = general_purpose::STANDARD.encode(auth.as_bytes());
        let auth_header = format!("Basic {}", auth_encoded);
        
        Ok(LeagueClient {
            client,
            base_url: format!("https://127.0.0.1:{}", port),
            auth_header,
            port,
        })
    }
    
    fn find_lockfile() -> Result<String, LeagueError> {
        let mut possible_paths = vec![
            "C:\\Riot Games\\League of Legends\\lockfile".to_string(),
            format!("{}/.local/share/applications/league-of-legends/lockfile", env::var("HOME").unwrap_or_default()),
        ];
        
        if cfg!(target_os = "macos") {
            let home = env::var("HOME").unwrap_or_default();
            possible_paths.extend(vec![
                "/Applications/League of Legends.app/Contents/LoL/lockfile".to_string(),
                format!("{}/Applications/League of Legends.app/Contents/LoL/lockfile", home),
                format!("{}/Library/Application Support/com.riotgames.league_of_legends.live/lockfile", home),
            ]);
        }
        
        for path in possible_paths {
            if Path::new(&path).exists() {
                return Ok(path);
            }
        }
        
        Err("League Client lockfile not found. Is League of Legends running?".into())
    }
    
    pub async fn is_in_ready_check(&self) -> Result<bool, LeagueError> {
        let url = format!("{}/lol-matchmaking/v1/ready-check", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            if let Some(state) = json.get("state") {
                return Ok(state == "InProgress");
            }
        }
        
        Ok(false)
    }
    
    pub async fn accept_ready_check(&self) -> Result<bool, LeagueError> {
        let url = format!("{}/lol-matchmaking/v1/ready-check/accept", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    pub async fn get_summoner_info(&self) -> Result<Value, LeagueError> {
        let url = format!("{}/lol-summoner/v1/current-summoner", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            Ok(json)
        } else {
            Err("Failed to get summoner info".into())
        }
    }
}

pub struct AutoAcceptService {
    client: LeagueClient,
}

impl AutoAcceptService {
    pub async fn new() -> Result<Self, LeagueError> {
        let client = LeagueClient::new().await?;
        Ok(AutoAcceptService {
            client,
        })
    }
    
    pub async fn start_monitoring(&mut self, app_handle: tauri::AppHandle) -> Result<(), LeagueError> {
        loop {
            match self.client.is_in_ready_check().await {
                Ok(true) => {
                    println!("Ready check detected! Auto-accepting...");
                    
                    match self.client.accept_ready_check().await {
                        Ok(true) => {
                            println!("Successfully accepted ready check!");
                            let _ = app_handle.emit("match-accepted", "Match accepted successfully!");
                        }
                        Ok(false) => {
                            println!("Failed to accept ready check");
                            let _ = app_handle.emit("match-accept-failed", "Failed to accept match");
                        }
                        Err(e) => {
                            println!("Error accepting ready check: {}", e);
                        }
                    }
                }
                Ok(false) => {
                }
                Err(e) => {
                    println!("Error checking ready check status: {}", e);
                    match LeagueClient::new().await {
                        Ok(new_client) => {
                            self.client = new_client;
                            println!("Reconnected to League Client");
                        }
                        Err(_) => {
                            let _ = app_handle.emit("league-disconnected", "League Client not found");
                            break;
                        }
                    }
                }
            }
            
            sleep(Duration::from_millis(1000)).await;
        }
        
        Ok(())
    }
}