use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::Once;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

// Global store for channel and global badges
static BADGES_INITIALIZED: Once = Once::new();
static BADGE_CACHE: Lazy<Arc<Mutex<BadgeCache>>> = 
    Lazy::new(|| Arc::new(Mutex::new(BadgeCache::new())));

// Structures for the badge API responses
#[derive(Debug, Deserialize)]
struct BadgeResponse {
    data: Vec<BadgeSet>,
}

#[derive(Debug, Deserialize)]
struct BadgeSet {
    set_id: String,
    versions: Vec<BadgeVersion>,
}

#[derive(Debug, Deserialize, Clone)]
struct BadgeVersion {
    id: String,
    image_url_1x: String,
    image_url_2x: String,
    image_url_4x: String,
    title: String,
    description: String,
}

// Our simplified badge structure that we'll send to the frontend
#[derive(Debug, Serialize, Clone)]
pub struct SimpleBadge {
    pub id: String,
    pub version: String,
    pub image_url: String,
    pub title: String,
}

// Badge cache to store all badges
#[derive(Debug)]
struct BadgeCache {
    channel_badges: HashMap<String, HashMap<String, BadgeVersion>>, // set_id -> id -> BadgeVersion
    global_badges: HashMap<String, HashMap<String, BadgeVersion>>,  // set_id -> id -> BadgeVersion
}

impl BadgeCache {
    fn new() -> Self {
        BadgeCache {
            channel_badges: HashMap::new(),
            global_badges: HashMap::new(),
        }
    }

    // Add channel badges to the cache
    fn add_channel_badges(&mut self, badges: Vec<BadgeSet>) {
        for badge_set in badges {
            let mut version_map = HashMap::new();
            for version in badge_set.versions {
                version_map.insert(version.id.clone(), version);
            }
            self.channel_badges.insert(badge_set.set_id, version_map);
        }
    }

    // Add global badges to the cache
    fn add_global_badges(&mut self, badges: Vec<BadgeSet>) {
        for badge_set in badges {
            let mut version_map = HashMap::new();
            for version in badge_set.versions {
                version_map.insert(version.id.clone(), version);
            }
            self.global_badges.insert(badge_set.set_id, version_map);
        }
    }

    // Get badge info based on set_id and id
    fn get_badge_info(&self, set_id: &str, id: &str) -> Option<SimpleBadge> {
        // Try to find in channel badges first
        if let Some(versions) = self.channel_badges.get(set_id) {
            if let Some(badge) = versions.get(id) {
                return Some(SimpleBadge {
                    id: set_id.to_string(),
                    version: id.to_string(),
                    image_url: badge.image_url_1x.clone(),
                    title: badge.title.clone(),
                });
            }
        }
        
        // If not found, try global badges
        if let Some(versions) = self.global_badges.get(set_id) {
            if let Some(badge) = versions.get(id) {
                return Some(SimpleBadge {
                    id: set_id.to_string(),
                    version: id.to_string(),
                    image_url: badge.image_url_1x.clone(),
                    title: badge.title.clone(),
                });
            }
        }
        
        None
    }
}

// Function to initialize badges
pub async fn initialize_badges(client_id: &str, token: &str, broadcaster_id: &str) -> Result<(), String> {
    let mut success = true;

    // Prepare headers for API requests
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Client-Id", HeaderValue::from_str(client_id).map_err(|e| e.to_string())?);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    
    // Get channel badges
    let channel_badges = match fetch_channel_badges(&client, &headers, broadcaster_id).await {
        Ok(badges) => badges,
        Err(e) => {
            println!("Error fetching channel badges: {}", e);
            success = false;
            Vec::new()
        }
    };
    
    // Get global badges
    let global_badges = match fetch_global_badges(&client, &headers).await {
        Ok(badges) => badges,
        Err(e) => {
            println!("Error fetching global badges: {}", e);
            success = false;
            Vec::new()
        }
    };
    
    // Store badges in cache
    if let Ok(mut cache) = BADGE_CACHE.lock() {
        cache.add_channel_badges(channel_badges);
        cache.add_global_badges(global_badges);
    } else {
        return Err("Failed to lock badge cache".to_string());
    }
    
    if success {
        Ok(())
    } else {
        Err("Failed to fetch all badges".to_string())
    }
}

// Function to fetch channel badges from Twitch API
async fn fetch_channel_badges(
    client: &reqwest::Client,
    headers: &HeaderMap,
    broadcaster_id: &str
) -> Result<Vec<BadgeSet>, String> {
    let url = format!("https://api.twitch.tv/helix/chat/badges?broadcaster_id={}", broadcaster_id);
    
    let response = client.get(&url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: Status code {}", response.status()));
    }
    
    let badge_response: BadgeResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(badge_response.data)
}

// Function to fetch global badges from Twitch API
async fn fetch_global_badges(
    client: &reqwest::Client,
    headers: &HeaderMap,
) -> Result<Vec<BadgeSet>, String> {
    let url = "https://api.twitch.tv/helix/chat/badges/global";
    
    let response = client.get(url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: Status code {}", response.status()));
    }
    
    let badge_response: BadgeResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(badge_response.data)
}

// Public function to convert incoming badges to SimpleBadges with URLs
pub fn process_message_badges(badges: &[twitch_eventsub::Badge]) -> Vec<SimpleBadge> {
    let mut result = Vec::new();
    
    if let Ok(cache) = BADGE_CACHE.lock() {
        for badge in badges {
            if let Some(badge_info) = cache.get_badge_info(&badge.set_id, &badge.id) {
                result.push(badge_info);
            } else {
                // Fallback to use the bare minimum we know about the badge
                result.push(SimpleBadge {
                    id: badge.id.clone(),
                    version: "1".to_string(), // Default to version 1
                    image_url: format!("https://static-cdn.jtvnw.net/badges/v1/{}/{}/1", badge.id, "1"),
                    title: badge.set_id.clone(),
                });
            }
        }
    }
    
    result
}

// Function to check if badges are initialized
pub fn ensure_badges_initialized(client_id: String, token: String, broadcaster_id: String) {
    BADGES_INITIALIZED.call_once(|| {
        // We need to spawn this as a task since it's async and call_once isn't
        tauri::async_runtime::spawn(async move {
            match initialize_badges(&client_id, &token, &broadcaster_id).await {
                Ok(_) => println!("Successfully initialized badges"),
                Err(e) => println!("Failed to initialize badges: {}", e),
            }
        });
    });
} 