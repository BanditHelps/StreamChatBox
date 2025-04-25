// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Manager};
use twitch_eventsub::*;
use tauri::async_runtime::spawn;
use std::time::Duration;
use std::sync::Once;

static START: Once = Once::new();

// Learn more about Tauri commands at https://v1.tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn test() -> String {
    "Best Chatter Ever".into()
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
                let responses = api.receive_all_messages(Some(Duration::from_millis(1)));
                for response in responses {
                    match response {
                        ResponseType::Event(Event::ChatMessage(md)) => {
                            println!("{} ({}): {}", md.chatter.name, md.colour, md.message.text);

                            let _ = app.emit_all("twitch-chat-message", serde_json::json!({
                                "user": md.chatter.name,
                                "color": md.colour,
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

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, start_twitch_listener])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
