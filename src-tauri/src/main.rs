#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod league_client;

use league_client::{AutoAcceptService, LeagueClient, ChampSelectConfig};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Manager, LogicalPosition, LogicalSize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

type ServiceState = Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>>; 
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
    println!("DEBUG: Updating config - auto_pick: {}, auto_ban: {}, pick_priority: {:?}, ban_priority: {:?}", 
             auto_pick_enabled, auto_ban_enabled, pick_priority, ban_priority);
    
    let mut config = config_state.lock().unwrap();
    config.auto_pick_enabled = auto_pick_enabled;
    config.auto_ban_enabled = auto_ban_enabled;
    config.pick_priority = pick_priority;
    config.ban_priority = ban_priority;
    
    println!("DEBUG: Config updated successfully");
    Ok("Configuration updated successfully".to_string())
}

#[tauri::command]
async fn get_champ_select_config(config_state: State<'_, ConfigState>) -> Result<ChampSelectConfig, String> {
    let config = config_state.lock().unwrap();
    println!("DEBUG: Retrieved config - auto_pick: {}, auto_ban: {}, pick_priority: {:?}, ban_priority: {:?}", 
             config.auto_pick_enabled, config.auto_ban_enabled, config.pick_priority, config.ban_priority);
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
        let current_service = service_state.lock().unwrap();
        if current_service.is_some() {
            return Err("Auto-accept is already running".to_string());
        }
    }
    
    let config = {
        let config_guard = config_state.lock().unwrap();
        let config = config_guard.clone();
        println!("DEBUG: Starting service with config - auto_pick: {}, auto_ban: {}", 
                 config.auto_pick_enabled, config.auto_ban_enabled);
        config
    };
    
    match AutoAcceptService::new().await {
        Ok(mut service) => {
            service.update_config(config);
            
            let service_state_clone = service_state.inner().clone();
            let config_state_clone = config_state.inner().clone();
            
            let handle = tauri::async_runtime::spawn(async move {
                let mut last_config: Option<ChampSelectConfig> = None;
                
                loop {
                    let current_config = {
                        let config_guard = config_state_clone.lock().unwrap();
                        config_guard.clone()
                    };
                    
                    let config_changed = match &last_config {
                        None => true,
                        Some(last) => last != &current_config
                    };
                    
                    if config_changed {
                        service.update_config(current_config.clone());
                        last_config = Some(current_config);
                    }
                    
                    match service.client.is_in_ready_check().await {
                        Ok(true) => {
                            println!("Ready check detected! Auto-accepting...");
                            
                            match service.client.accept_ready_check().await {
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
                            if let Err(e) = service.handle_champion_select(&app_handle).await {
                                println!("Champion select error: {}", e);
                            }
                        }
                        Err(e) => {
                            println!("Error checking ready check status: {}", e);
                            match LeagueClient::new().await {
                                Ok(new_client) => {
                                    service.client = new_client;
                                    println!("Reconnected to League Client");
                                }
                                Err(_) => {
                                    let _ = app_handle.emit("league-disconnected", "League Client not found");
                                    break;
                                }
                            }
                        }
                    }
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
                
                {
                    let mut current_service = service_state_clone.lock().unwrap();
                    *current_service = None;
                }
            });
            
            {
                let mut current_service = service_state.lock().unwrap();
                *current_service = Some(handle);
            }
            
            Ok("Auto-accept started successfully".to_string())
        }
        Err(e) => Err(format!("Failed to start auto-accept: {}", e))
    }
}

#[tauri::command]
async fn stop_auto_accept(service_state: State<'_, ServiceState>) -> Result<String, String> {
    let mut current_service = service_state.lock().unwrap();
    
    if let Some(handle) = current_service.take() {
        handle.abort();
        Ok("Auto-accept stopped successfully".to_string())
    } else {
        Err("Auto-accept is not running".to_string())
    }
}

#[tauri::command]
async fn is_auto_accept_running(service_state: State<'_, ServiceState>) -> Result<bool, String> {
    let current_service = service_state.lock().unwrap();
    Ok(current_service.is_some())
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

#[tauri::command]
async fn show_window(app_handle: tauri::AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        
        if let Ok(monitor) = window.current_monitor() {
            if let Some(monitor) = monitor {
                let monitor_size = monitor.size();
                let window_size = LogicalSize::new(900, 580); // Updated to match new height
                
                let position = LogicalPosition::new(
                    monitor_size.width as i32 - window_size.width - 100,
                    50 
                );
                let _ = window.set_position(position);
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app_handle: tauri::AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ServiceState::new(Mutex::new(None)))
        .manage(ConfigState::new(Mutex::new(ChampSelectConfig::default())))
        .invoke_handler(tauri::generate_handler![
            connect_to_league,
            update_champ_select_config,
            get_champ_select_config,
            get_all_champions,
            start_auto_accept,
            stop_auto_accept,
            is_auto_accept_running,
            manual_accept,
            show_window,
            hide_window
        ])
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Lolytics Auto Accept")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            let app_handle = app.clone();
                            let _ = tauri::async_runtime::spawn(async move {
                                let _ = show_window(app_handle).await;
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                            let app_handle = tray.app_handle().clone();
                            let _ = tauri::async_runtime::spawn(async move {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = hide_window(app_handle).await;
                                    } else {
                                        let _ = show_window(app_handle).await;
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                })
                .build(app)?;

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