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
use std::thread;

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

            let twitch = TwitchEventSubApi::builder(keys)
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

                            let _ = app.emit_all("twitch-chat-message", serde_json::json!({
                                "user": md.chatter.name,
                                "color": color,
                                "message": md.message.text,
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet, 
            start_twitch_listener, 
            send_chat_message,
            start_mock_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}