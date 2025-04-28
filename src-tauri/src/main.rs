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
use std::{env, fs, error::Error};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use colored::Colorize;
use chrono::{DateTime, Local};

// Add the badges module
mod badges;

static START: Once = Once::new();
static MOCK_START: Once = Once::new();
static YOUTUBE_START: Once = Once::new();
// Message queue for chat messages
static TWITCH_MESSAGE_QUEUE: once_cell::sync::Lazy<Arc<Mutex<VecDeque<String>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

static YOUTUBE_MESSAGE_QUEUE: once_cell::sync::Lazy<Arc<Mutex<VecDeque<String>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

// Data Structures for response from APIs
// YouTube API response structures
#[derive(Debug, Deserialize)]
struct LiveChatResponse {
    items: Option<Vec<ChatMessage>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "pollingIntervalMillis")]
    polling_interval_millis: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    #[serde(rename = "snippet")]
    snippet: MessageSnippet,
    #[serde(rename = "authorDetails")]
    author_details: AuthorDetails,
}

#[derive(Debug, Deserialize)]
struct MessageSnippet {
    #[serde(rename = "displayMessage")]
    display_message: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}

#[derive(Debug, Deserialize)]
struct AuthorDetails {
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "isChatOwner")]
    is_owner: Option<bool>,
    #[serde(rename = "isChatModerator")]
    is_moderator: Option<bool>,
    #[serde(rename = "isChatSponsor")]
    is_sponsor: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    #[serde(rename = "id")]
    id: SearchItemId,
}

#[derive(Debug, Deserialize)]
struct SearchItemId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LiveVideoResponse {
    items: Vec<LiveVideoItem>,
}

#[derive(Debug, Deserialize)]
struct LiveVideoItem {
    #[serde(rename = "liveStreamingDetails")]
    live_streaming_details: Option<LiveStreamingDetails>,
}

#[derive(Debug, Deserialize)]
struct LiveStreamingDetails {
    #[serde(rename = "activeLiveChatId")]
    active_live_chat_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    snippet: Snippet,
}

#[derive(Debug, Serialize)]
struct Snippet {
    liveChatId: String,
    #[serde(rename = "type")]
    type_field: String,  // `type` is a Rust keyword, so we rename it
    textMessageDetails: TextMessageDetails,
}

#[derive(Debug, Serialize)]
struct TextMessageDetails {
    messageText: String,
}




async fn get_live_video_id(client: &Client, channel_id: &str, api_key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let search_url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=id&eventType=live&type=video&channelId={}&key={}",
        channel_id, api_key
    );
    
    let search_response = client.get(&search_url)
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;
    
    if search_response.items.is_empty() {
        return Err("No live streams found for this channel".into());
    }
    
    match &search_response.items[0].id.video_id {
        Some(video_id) => Ok(video_id.clone()),
        None => Err("Could not find video ID in the search response".into()),
    }
}

async fn get_live_chat_id(client: &Client, video_id: &str, api_key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let video_url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=liveStreamingDetails&id={}&key={}",
        video_id, api_key
    );
    
    let video_response = client.get(&video_url)
        .send()
        .await?
        .json::<LiveVideoResponse>()
        .await?;
    
    if video_response.items.is_empty() {
        return Err("No video details found".into());
    }
    
    match &video_response.items[0].live_streaming_details {
        Some(details) => {
            match &details.active_live_chat_id {
                Some(chat_id) => Ok(chat_id.clone()),
                None => Err("No active live chat found for this video".into()),
            }
        },
        None => Err("No live streaming details found for this video".into()),
    }
}

async fn youtube_send_chat(client: &Client, video_id: &str, message: &str) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/liveChat/messages?part=snippet"
    );

    println!("YouTube chat URL: {}", url);

    let request = SendMessageRequest {
        snippet: Snippet {
            liveChatId: video_id.to_string(),
            type_field: "textMessageEvent".to_string(),
            textMessageDetails: TextMessageDetails {
                messageText: message.to_string(),
            },
        },
    };

    println!("YouTube chat request: {:?}", request);

    let response = client.get(&url)
        .json(&request)
        .send()
        .await?;

    // Rprint the response to see if it is bad
    println!("YouTube chat response: {:?}", response);
    Ok(response)

}


async fn fetch_chat_messages(
    client: &Client, 
    chat_id: &str, 
    api_key: &str, 
    next_page_token: Option<&str>
) -> Result<LiveChatResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut url = format!(
        "https://www.googleapis.com/youtube/v3/liveChat/messages?liveChatId={}&part=snippet,authorDetails&key={}",
        chat_id, api_key
    );
    
    if let Some(token) = next_page_token {
        url.push_str(&format!("&pageToken={}", token));
    }
    
    let response = client.get(&url)
        .send()
        .await?
        .json::<LiveChatResponse>()
        .await?;


    Ok(response)
}

fn format_timestamp(timestamp_str: &str) -> Result<String, Box<dyn Error>> {
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;
    // let timestamp_utc: DateTime<Utc> = timestamp.into();
    let timestamp_local = timestamp.with_timezone(&Local);
    Ok(timestamp_local.format("%H:%M:%S").to_string())
}


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
    {
        let mut twitch_queue = TWITCH_MESSAGE_QUEUE.lock()
            .map_err(|e| format!("Failed to lock Twitch queue: {}", e))?;
        twitch_queue.push_back(message.clone());
    }

    {
        let mut youtube_queue = YOUTUBE_MESSAGE_QUEUE.lock()
            .map_err(|e| format!("Failed to lock YouTube queue: {}", e))?;
        youtube_queue.push_back(message);
    }

    Ok(())
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
fn start_youtube_listener(app: AppHandle) {
    let app_clone = app.clone();
    YOUTUBE_START.call_once(move || {
        let app_clone2 = app_clone.clone();
        tauri::async_runtime::spawn(async move {
            simple_env_load::load_env_from([".secrets.env"]);

            fn get(key: &str) -> Result<String, String> {
                std::env::var(key).map_err(|_| format!("please set {key} in .example.env"))
            }

            let youtube_channel_id = match get("YOUTUBE_CHANNEL_ID") {
                Ok(s) => s,
                Err(e) => {
                    error!("{}", e);
                    String::from("default")
                }
            };

            let youtube_api_key = match get("YOUTUBE_API_KEY") {
                Ok(s) => s,
                Err(e) => {
                    error!("{}", e);
                    String::from("default")
                }
            };

            println!("{}", "Starting YouTube Listener".red().bold());

            let client = Client::new();
            
            match async_youtube_listener(client, youtube_channel_id, youtube_api_key, app_clone2).await {
                Ok(_) => println!("YouTube listener finished successfully"),
                Err(e) => println!("YouTube listener error: {}", e),
            }
        });
    });
}

// Separate async function to handle all the YouTube listener logic
async fn async_youtube_listener(
    client: Client,
    youtube_channel_id: String,
    youtube_api_key: String,
    app: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let video_id = get_live_video_id(&client, &youtube_channel_id, &youtube_api_key).await?;
    let chat_id = get_live_chat_id(&client, &video_id, &youtube_api_key).await?;

    let mut next_token: Option<String> = None;

    println!("{}", "YouTube Setup successful".red());
    
    loop {
        let response = fetch_chat_messages(
            &client, 
            &chat_id, 
            &youtube_api_key, 
            next_token.as_deref()
        ).await?;

        if response.items.is_none() {
            println!("{}", "No chat detected. Continuing".red());
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        for message in &response.items.unwrap() {
            let timestamp = match format_timestamp(&message.snippet.published_at) {
                Ok(ts) => ts,
                Err(_) => "??:??:??".to_string()
            };
            
            let username = if message.author_details.is_owner.unwrap_or(false) {
                message.author_details.display_name.red().bold()
            } else if message.author_details.is_moderator.unwrap_or(false) {
                message.author_details.display_name.blue().bold()
            } else if message.author_details.is_sponsor.unwrap_or(false) {
                message.author_details.display_name.green().bold()
            } else {
                message.author_details.display_name.yellow()
            };
            
            println!("[{}] {}: {}", 
                timestamp.bright_black(), 
                username, 
                message.snippet.display_message);

            let _ = app.emit_all("youtube-chat-message", serde_json::json!({
                "user": message.author_details.display_name,
                "color": get_random_color(),
                "message": message.snippet.display_message,
                "timestamp": timestamp
            }));
        }

        if response.polling_interval_millis.is_some() {
            next_token = response.next_page_token.clone();
        }

        // Process outgoing messages
        let messages = {
            let mut queue = YOUTUBE_MESSAGE_QUEUE.lock().unwrap();
            let msgs = queue.drain(..).collect::<Vec<_>>();
            msgs // Return msgs outside of the lock scope
        };
        
        // Process each message after the lock is released
        for message in messages {
            match youtube_send_chat(&client, &video_id, &message).await {
                Ok(_) => println!("Sent YouTube chat message: {}", message),
                Err(e) => println!("Failed to send YouTube chat message: {}", e)
            }
        }
        
        if response.polling_interval_millis.is_some() {
            tokio::time::sleep(Duration::from_millis(response.polling_interval_millis.unwrap())).await;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        } 
    }
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
                if let Ok(mut queue) = TWITCH_MESSAGE_QUEUE.lock() {
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
            start_youtube_listener,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}