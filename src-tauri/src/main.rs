#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod league_client;

use league_client::{AutoAcceptService, LeagueClient, ChampSelectConfig};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

type ServiceState = Arc<Mutex<bool>>; 
type ConfigState = Arc<Mutex<ChampSelectConfig>>;

#[tauri::command]
async fn connect_to_league() -> Result<String, String> {
    match LeagueClient::new().await {
        Ok(client) => {
            match client.get_summoner_info().await {
                Ok(summoner_info) => {
                    let display_name = summoner_info
                        .get("displayName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    Ok(format!("Connected to League Client. Summoner: {}", display_name))
                }
                Err(e) => Err(format!("Connected to League Client but failed to get summoner info: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to connect to League Client: {}", e))
    }
}

#[tauri::command]
async fn update_champ_select_config(
    config_state: State<'_, ConfigState>,
    auto_pick_enabled: bool,
    auto_ban_enabled: bool,
    pick_priority: Vec<String>,
    ban_priority: Vec<String>,
) -> Result<String, String> {
    let mut config = config_state.lock().unwrap();
    config.auto_pick_enabled = auto_pick_enabled;
    config.auto_ban_enabled = auto_ban_enabled;
    config.pick_priority = pick_priority;
    config.ban_priority = ban_priority;
    
    Ok("Configuration updated successfully".to_string())
}

#[tauri::command]
async fn get_champ_select_config(config_state: State<'_, ConfigState>) -> Result<ChampSelectConfig, String> {
    let config = config_state.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
async fn get_all_champions() -> Result<Vec<String>, String> {
    match LeagueClient::new().await {
        Ok(client) => {
            match client.get_all_champion_names().await {
                Ok(champions) => Ok(champions),
                Err(e) => Err(format!("Failed to get champions: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to connect to League Client: {}", e))
    }
}

#[tauri::command]
async fn start_auto_accept(
    service_state: State<'_, ServiceState>,
    config_state: State<'_, ConfigState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    {
        let is_running = service_state.lock().unwrap();
        if *is_running {
            return Err("Auto-accept is already running".to_string());
        }
    }
    
    let config = {
        let config_guard = config_state.lock().unwrap();
        config_guard.clone()
    };
    
    match AutoAcceptService::new().await {
        Ok(mut service) => {
            service.update_config(config);
            
            {
                let mut is_running = service_state.lock().unwrap();
                *is_running = true;
            }
            
            let service_state_clone = service_state.inner().clone();
            
            tauri::async_runtime::spawn(async move {
                if let Err(e) = service.start_monitoring(app_handle).await {
                    println!("Auto-accept service error: {}", e);
                }
                
                {
                    let mut is_running = service_state_clone.lock().unwrap();
                    *is_running = false;
                }
            });
            
            Ok("Auto-accept started successfully".to_string())
        }
        Err(e) => Err(format!("Failed to start auto-accept: {}", e))
    }
}

#[tauri::command]
async fn stop_auto_accept(service_state: State<'_, ServiceState>) -> Result<String, String> {
    let mut is_running = service_state.lock().unwrap();
    
    if *is_running {
        *is_running = false;
        Ok("Auto-accept will stop after current check".to_string())
    } else {
        Err("Auto-accept is not running".to_string())
    }
}

#[tauri::command]
async fn is_auto_accept_running(service_state: State<'_, ServiceState>) -> Result<bool, String> {
    let is_running = service_state.lock().unwrap();
    Ok(*is_running)
}

#[tauri::command]
async fn manual_accept() -> Result<String, String> {
    match LeagueClient::new().await {
        Ok(client) => {
            match client.accept_ready_check().await {
                Ok(true) => Ok("Match accepted successfully!".to_string()),
                Ok(false) => Err("Failed to accept match (no ready check active?)".to_string()),
                Err(e) => Err(format!("Error accepting match: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to connect to League Client: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ServiceState::new(Mutex::new(false)))
        .manage(ConfigState::new(Mutex::new(ChampSelectConfig::default())))
        .invoke_handler(tauri::generate_handler![
            connect_to_league,
            update_champ_select_config,
            get_champ_select_config,
            get_all_champions,
            start_auto_accept,
            stop_auto_accept,
            is_auto_accept_running,
            manual_accept
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let _ = app_handle.emit("app-ready", ());
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}