// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Manager};
use twitch_eventsub::*;
use std::time::Duration;
use std::sync::Once;
use random_color::RandomColor;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::time::sleep;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

// Add the badges module
mod badges;

static START: Once = Once::new();
static MOCK_START: Once = Once::new();
// Message queue for chat messages
static MESSAGE_QUEUE: once_cell::sync::Lazy<Arc<Mutex<VecDeque<String>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

// Learn more about Tauri commands at https://v1.tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Return a random color hex code
fn get_random_color() -> String {
    return RandomColor::new().to_hex();
}

// New function to queue chat messages from frontend
#[tauri::command]
fn send_chat_message(message: String) -> Result<(), String> {
    match MESSAGE_QUEUE.lock() {
        Ok(mut queue) => {
            queue.push_back(message);
            Ok(())
        },
        Err(e) => Err(format!("Failed to queue message: {}", e))
    }
}

// Command to initialize badges
#[tauri::command]
fn initialize_twitch_badges(client_id: String, access_token: String, broadcaster_id: String) -> Result<(), String> {
    println!("Initializing Twitch badges...");
    badges::ensure_badges_initialized(client_id, access_token, broadcaster_id);
    Ok(())
}

// Command to initialize badges from environment files
#[tauri::command]
fn initialize_badges_from_env() -> Result<(), String> {
    println!("Initializing Twitch badges from environment files (frontend request)...");
    
    // Spawn the async task
    tauri::async_runtime::spawn(async {
        match initialize_badges_from_env_internal().await {
            Ok(_) => println!("Badge initialization from frontend successful"),
            Err(e) => println!("Badge initialization from frontend failed: {}", e),
        }
    });
    
    // Return success immediately since we're doing this asynchronously
    Ok(())
}

// Start a mock events generator for testing donation and subscription events
#[tauri::command]
fn start_mock_events(app: AppHandle) {
    MOCK_START.call_once(|| {
        tauri::async_runtime::spawn(async move {
            println!("Started mock events generator...");
            
            loop {
                // Random delay between events (5-15 seconds)
                let delay = rand::random::<u64>() % 10 + 5;
                sleep(Duration::from_secs(delay)).await;
                
                // Randomly select event type
                let event_type = rand::random::<u8>() % 3;
                
                match event_type {
                    0 => {
                        // Generate mock donation
                        let amount = (rand::random::<f64>() * 100.0).round() / 100.0;
                        let username = format!("Donor{}", rand::random::<u16>() % 1000);
                        let message = if rand::random::<bool>() {
                            Some(format!("Thanks for the stream! Keep up the good work!"))
                        } else {
                            None
                        };
                        
                        println!("{} donated ${:.2}!", username, amount);
                        
                        let _ = app.emit_all("twitch-donation", serde_json::json!({
                            "username": username,
                            "amount": amount,
                            "message": message,
                        }));
                    },
                    1 => {
                        // Generate mock subscription
                        let username = format!("Sub{}", rand::random::<u16>() % 1000);
                        let tier = rand::random::<u8>() % 3 + 1;
                        let is_gift = rand::random::<bool>();
                        
                        println!("{} subscribed with tier {}!", username, tier);
                        
                        let _ = app.emit_all("twitch-subscription", serde_json::json!({
                            "username": username,
                            "tier": tier,
                            "is_gift": is_gift,
                        }));
                    },
                    _ => {
                        // Generate mock follow (already handled by the real system, 
                        // but we'll add extra ones for testing)
                        let username = format!("Follower{}", rand::random::<u16>() % 1000);
                        
                        println!("{} followed!", username);
                        
                        let _ = app.emit_all("twitch-follow", serde_json::json!({
                            "user": username
                        }));
                    }
                }
            }
        });
    });
}

#[tauri::command]
fn start_twitch_listener(app: AppHandle) {
    START.call_once(|| {
        tauri::async_runtime::spawn(async move {
            let keys = TwitchKeys::from_secrets_env()
                .expect("Set TWITCH_* env vars");

            let twitch = TwitchEventSubApi::builder(keys.clone())
                .set_redirect_url("http://localhost:3001")
                .generate_new_token_if_none(true)
                .generate_access_token_on_expire(true)
                .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
                .add_subscriptions(vec![
                    Subscription::ChatMessage,
                    Subscription::ChannelFollow
                ]);

            let mut api = twitch.build()
                .expect("Failed to build EventSub API");

            println!("Started Twitch Monitoring...");
            
            // Initialize badges after API is built and token is available
            initialize_badges_after_api_built(&app, keys).await;
            
            loop {
                // Process incoming messages
                let responses = api.receive_all_messages(Some(Duration::from_millis(1)));
                for response in responses {
                    match response {
                        ResponseType::Event(Event::ChatMessage(md)) => {
                            println!("{} ({}): {}", md.chatter.name, md.colour, md.message.text);

                            let mut color = md.colour; 

                            if color == "" {
                                // color = String::from("#ffffff");
                                color = get_random_color();
                            }
                            
                            // Process the badges to get URLs
                            let processed_badges = badges::process_message_badges(&md.badges);
                            
                            // Convert processed badges to a format suitable for JSON
                            let badge_data = processed_badges.into_iter().map(|badge| {
                                serde_json::json!({
                                    "id": badge.id,
                                    "version": badge.version,
                                    "image_url": badge.image_url,
                                    "title": badge.title
                                })
                            }).collect::<Vec<_>>();

                            let _ = app.emit_all("twitch-chat-message", serde_json::json!({
                                "user": md.chatter.name,
                                "color": color,
                                "message": md.message.text,
                                "badges": badge_data,
                            }));
                           },
                        ResponseType::Event(Event::Follow(fd)) => {
                            println!("{} followed on Twitch!", fd.user.name);

                            let _ = app.emit_all("twitch-follow", serde_json::json!({
                                "user": fd.user.name
                            }));
                        },
                        _ => {}
                    }
                }

                // Send any queued messages
                if let Ok(mut queue) = MESSAGE_QUEUE.lock() {
                    while let Some(message) = queue.pop_front() {
                        match api.send_chat_message(&message) {
                            Ok(_) => println!("Sent chat message: {}", message),
                            Err(_e) => println!("Failed to send chat message")
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    });
}

// Helper function to initialize badges after API is built
async fn initialize_badges_after_api_built(app: &AppHandle, keys: TwitchKeys) {
    // Wait a moment to ensure token files are written
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    println!("Attempting to initialize badges after Twitch API startup");
    
    // Get client_id directly from keys (it's not an Option)
    let client_id = keys.client_id;
    
    // Get broadcaster_id directly from keys (it's not an Option)
    let broadcaster_id = keys.broadcaster_account_id;
    
    // For the token, we still need to read from file as TwitchKeys.access_token
    // is an Option<TokenAccess> which doesn't directly give us the string we need
    let access_token = match std::fs::read_to_string(".user_token.env") {
        Ok(content) => {
            let token = content.trim();
            if token.is_empty() {
                let error = "Token file is empty";
                println!("{}", error);
                let _ = app.emit_all("badges-initialization-failed", error);
                return;
            }
            token.to_string()
        },
        Err(e) => {
            let error = format!("Error reading token file: {}", e);
            println!("{}", error);
            let _ = app.emit_all("badges-initialization-failed", error);
            return;
        }
    };
    
    println!("Using keys directly - client_id length: {}, broadcaster_id: {}, token length: {}", 
            client_id.len(), broadcaster_id, access_token.len());
    
    // Initialize badges
    match badges::initialize_badges(&client_id, &access_token, &broadcaster_id).await {
        Ok(_) => {
            println!("Successfully initialized badges after API startup");
            // Emit an event to the frontend to notify that badges are ready
            let _ = app.emit_all("badges-initialized", true);
        },
        Err(e) => {
            println!("Failed to initialize badges after API startup: {}", e);
            // Emit an event to the frontend to notify that badge initialization failed
            let _ = app.emit_all("badges-initialization-failed", e);
        }
    }
}

// Internal function that implements the badge initialization logic
async fn initialize_badges_from_env_internal() -> Result<(), String> {
    // Get client ID from .secrets.env
    let client_id = match env::var("TWITCH_CLIENT_ID") {
        Ok(id) => id,
        Err(_) => return Err("TWITCH_CLIENT_ID not found in environment".to_string()),
    };
    
    // Get broadcaster ID from .secrets.env
    let broadcaster_id = match env::var("TWITCH_CHANNEL_ID") {
        Ok(id) => id,
        Err(_) => return Err("TWITCH_CHANNEL_ID not found in environment".to_string()),
    };
    
    // Try to read access token from .user_token.env
    let access_token = match std::fs::read_to_string(".user_token.env") {
        Ok(content) => {
            println!("Found user token file");
            let token = content.trim();
            if token.is_empty() {
                return Err("Token file is empty".to_string());
            }
            token.to_string()
        },
        Err(e) => {
            println!("Error reading user token file: {}", e);
            // Try loading .secrets.env for access token
            match env::var("TWITCH_USER_TOKEN") {
                Ok(token) => {
                    println!("Using TWITCH_USER_TOKEN from environment");
                    token
                },
                Err(_) => {
                    return Err(format!("Could not read access token from any source: {}", e));
                }
            }
        }
    };
    
    println!("Using client_id length: {}, broadcaster_id: {}, token length: {}", 
            client_id.len(), broadcaster_id, access_token.len());
    
    // Initialize badges
    badges::initialize_badges(&client_id, &access_token, &broadcaster_id).await
}

// Command to save API keys to a secure file
#[tauri::command]
fn save_api_keys(
    app_handle: tauri::AppHandle,
    twitch_client_id: String,
    twitch_client_secret: String,
    twitch_broadcaster_id: String,
    youtube_channel_id: String,
    youtube_api_key: String,
) -> Result<(), String> {
    let contents = format!(
        "TWITCH_CLIENT_ID={}\nTWITCH_CLIENT_SECRET={}\nTWITCH_BROADCASTER_ID={}\nYOUTUBE_CHANNEL_ID={}\nYOUTUBE_API_KEY={}",
        twitch_client_id, twitch_client_secret, twitch_broadcaster_id, youtube_channel_id, youtube_api_key
    );
    
    // Use the app's resource path for better security
    let resource_path = app_handle.path_resolver().app_config_dir()
        .ok_or_else(|| "Could not find config directory".to_string())?;
    
    // Create directory if it doesn't exist
    match fs::create_dir_all(&resource_path) {
        Ok(_) => {},
        Err(e) => return Err(format!("Failed to create config directory: {}", e))
    }
    
    let secrets_path = resource_path.join(".secrets.env");
    
    match fs::write(&secrets_path, contents) {
        Ok(_) => {
            // Also write to current directory for compatibility with existing code
            // match fs::write("./.secrets.env", contents) {
            //     Ok(_) => Ok(()),
            //     Err(e) => Err(format!("Failed to save API keys to current directory: {}", e))
            // }
            println!("Saved API keys to {}", secrets_path.display());
            Ok(())
        },
        Err(e) => Err(format!("Failed to save API keys: {}", e))
    }
}

// Command to read API keys from the secure file
#[tauri::command]
fn read_api_keys(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    // Try app config directory first
    let resource_path = app_handle.path_resolver().app_config_dir()
        .ok_or_else(|| "Could not find config directory".to_string())?;
    
    let secrets_path = resource_path.join(".secrets.env");
    
    let path = if secrets_path.exists() {
        secrets_path
    } else {
        // Fallback to current directory
        PathBuf::from("./.secrets.env")
    };
    
    if !path.exists() {
        return Ok(serde_json::json!({
            "TWITCH_CLIENT_ID": "",
            "TWITCH_CLIENT_SECRET": "",
            "TWITCH_BROADCASTER_ID": "",
            "YOUTUBE_CHANNEL_ID": "",
            "YOUTUBE_API_KEY": ""
        }));
    }
    
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open secrets file: {}", e))
    };
    
    let reader = BufReader::new(file);
    let mut api_keys = serde_json::json!({
        "TWITCH_CLIENT_ID": "",
        "TWITCH_CLIENT_SECRET": "",
        "TWITCH_BROADCASTER_ID": "",
        "YOUTUBE_CHANNEL_ID": "",
        "YOUTUBE_API_KEY": ""
    });
    
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => return Err(format!("Failed to read line: {}", e))
        };
        
        if let Some((key, value)) = line.split_once('=') {
            if !value.is_empty() {
                api_keys[key] = serde_json::Value::String(value.to_string());
            }
        }
    }
    
    Ok(api_keys)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet, 
            start_twitch_listener, 
            send_chat_message,
            start_mock_events,
            initialize_twitch_badges,
            initialize_badges_from_env,
            save_api_keys,
            read_api_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}