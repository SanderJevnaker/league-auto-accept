use reqwest::Client;
use serde_json::{Value, json};
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChampSelectConfig {
    pub auto_pick_enabled: bool,
    pub auto_ban_enabled: bool,
    pub pick_priority: Vec<String>,
    pub ban_priority: Vec<String>, 
}

impl Default for ChampSelectConfig {
    fn default() -> Self {
        Self {
            auto_pick_enabled: false,
            auto_ban_enabled: false,
            pick_priority: vec!["Jinx".to_string(), "Ashe".to_string(), "Caitlyn".to_string()],
            ban_priority: vec!["Yasuo".to_string(), "Zed".to_string(), "Master Yi".to_string()],
        }
    }
}

#[derive(Debug)]
pub struct ChampSelectPhase {
    pub phase: String,
    pub is_in_progress: bool,
    pub local_player_cell_id: i64,
    pub actions: Vec<Value>,
}

pub struct LeagueClient {
    client: Client,
    base_url: String,
    auth_header: String,
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
    
    pub async fn get_champ_select_session(&self) -> Result<Option<Value>, LeagueError> {
        let url = format!("{}/lol-champ-select/v1/session", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            Ok(Some(json))
        } else if response.status().as_u16() == 404 {
            Ok(None) 
        } else {
            Err("Failed to get champion select session".into())
        }
    }
    
    pub async fn get_available_champions(&self) -> Result<Vec<Value>, LeagueError> {
        let url = format!("{}/lol-champions/v1/owned-champions-minimal", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Vec<Value> = response.json().await?;
            Ok(json)
        } else {
            Err("Failed to get available champions".into())
        }
    }
    
    pub async fn pick_champion(&self, action_id: i64, champion_id: i64) -> Result<bool, LeagueError> {
        let url = format!("{}/lol-champ-select/v1/session/actions/{}", self.base_url, action_id);
        
        let payload = json!({
            "championId": champion_id,
            "completed": true,
            "type": "pick"
        });
        
        let response = self.client
            .patch(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    pub async fn ban_champion(&self, action_id: i64, champion_id: i64) -> Result<bool, LeagueError> {
        let url = format!("{}/lol-champ-select/v1/session/actions/{}", self.base_url, action_id);
        
        let payload = json!({
            "championId": champion_id,
            "completed": true,
            "type": "ban"
        });
        
        let response = self.client
            .patch(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    pub async fn get_all_champion_names(&self) -> Result<Vec<String>, LeagueError> {
        let url = format!("{}/lol-game-data/assets/v1/champions.json", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            let mut champion_names = Vec::new();
            
            if let Some(champions) = json.as_object() {
                for (_, champion_data) in champions {
                    if let Some(name) = champion_data.get("name").and_then(|n| n.as_str()) {
                        champion_names.push(name.to_string());
                    }
                }
            }
            
            champion_names.sort();
            Ok(champion_names)
        } else {
            Err("Failed to get champion data".into())
        }
    }
    
    pub async fn get_champion_id_by_name(&self, champion_name: &str) -> Result<Option<i64>, LeagueError> {
        let url = format!("{}/lol-game-data/assets/v1/champions.json", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            
            if let Some(champions) = json.as_object() {
                for (_, champion_data) in champions {
                    if let Some(name) = champion_data.get("name").and_then(|n| n.as_str()) {
                        if name.to_lowercase() == champion_name.to_lowercase() {
                            if let Some(id) = champion_data.get("id").and_then(|i| i.as_i64()) {
                                return Ok(Some(id));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
}

pub struct AutoAcceptService {
    client: LeagueClient,
    config: ChampSelectConfig,
}

impl AutoAcceptService {
    pub async fn new() -> Result<Self, LeagueError> {
        let client = LeagueClient::new().await?;
        Ok(AutoAcceptService {
            client,
            config: ChampSelectConfig::default(),
        })
    }
    
    pub fn update_config(&mut self, config: ChampSelectConfig) {
        self.config = config;
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
                    if let Err(e) = self.handle_champion_select(&app_handle).await {
                        println!("Champion select error: {}", e);
                    }
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
        
        Ok(())
    }
    
    async fn handle_champion_select(&self, app_handle: &tauri::AppHandle) -> Result<(), LeagueError> {
        if let Some(session) = self.client.get_champ_select_session().await? {
            let local_player_cell_id = session.get("localPlayerCellId")
                .and_then(|id| id.as_i64())
                .unwrap_or(-1);
            
            if let Some(actions) = session.get("actions").and_then(|a| a.as_array()) {
                for action_group in actions {
                    if let Some(action_array) = action_group.as_array() {
                        for action in action_array {
                            let actor_cell_id = action.get("actorCellId").and_then(|id| id.as_i64()).unwrap_or(-1);
                            let action_type = action.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            let is_in_progress = action.get("isInProgress").and_then(|p| p.as_bool()).unwrap_or(false);
                            let completed = action.get("completed").and_then(|c| c.as_bool()).unwrap_or(false);
                            let action_id = action.get("id").and_then(|id| id.as_i64()).unwrap_or(-1);
                            
                            if actor_cell_id == local_player_cell_id && is_in_progress && !completed {
                                match action_type {
                                    "ban" if self.config.auto_ban_enabled => {
                                        if let Err(e) = self.handle_auto_ban(action_id, app_handle).await {
                                            println!("Auto-ban error: {}", e);
                                        }
                                    }
                                    "pick" if self.config.auto_pick_enabled => {
                                        if let Err(e) = self.handle_auto_pick(action_id, app_handle).await {
                                            println!("Auto-pick error: {}", e);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_auto_ban(&self, action_id: i64, app_handle: &tauri::AppHandle) -> Result<(), LeagueError> {
        for champion_name in &self.config.ban_priority {
            if let Some(champion_id) = self.client.get_champion_id_by_name(champion_name).await? {
                match self.client.ban_champion(action_id, champion_id).await {
                    Ok(true) => {
                        println!("Successfully banned {}", champion_name);
                        let _ = app_handle.emit("champion-banned", format!("Banned {}", champion_name));
                        return Ok(());
                    }
                    Ok(false) => {
                        println!("Failed to ban {} (might be already banned)", champion_name);
                        continue;
                    }
                    Err(e) => {
                        println!("Error banning {}: {}", champion_name, e);
                        continue;
                    }
                }
            }
        }
        
        let _ = app_handle.emit("champion-ban-failed", "No champions from ban list available");
        Ok(())
    }
    
    async fn handle_auto_pick(&self, action_id: i64, app_handle: &tauri::AppHandle) -> Result<(), LeagueError> {
        let available_champions = self.client.get_available_champions().await?;
        
        for champion_name in &self.config.pick_priority {
            if let Some(champion_id) = self.client.get_champion_id_by_name(champion_name).await? {
                let is_owned = available_champions.iter().any(|champ| {
                    champ.get("id").and_then(|id| id.as_i64()) == Some(champion_id)
                });
                
                if is_owned {
                    match self.client.pick_champion(action_id, champion_id).await {
                        Ok(true) => {
                            println!("Successfully picked {}", champion_name);
                            let _ = app_handle.emit("champion-picked", format!("Picked {}", champion_name));
                            return Ok(());
                        }
                        Ok(false) => {
                            println!("Failed to pick {} (might be banned/picked)", champion_name);
                            continue;
                        }
                        Err(e) => {
                            println!("Error picking {}: {}", champion_name, e);
                            continue;
                        }
                    }
                }
            }
        }
        
        let _ = app_handle.emit("champion-pick-failed", "No champions from pick list available");
        Ok(())
    }
}