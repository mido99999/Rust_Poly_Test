use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const CLOB_WSS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Monitor CLOB WebSocket for the next 15-minute BTC market
pub async fn monitor_next_market_websocket() -> Result<(), Box<dyn std::error::Error >> {
    println!("🚀 CLOB WebSocket Monitor Started\n");
    
    // Calculate the next market slug we're looking for
    let next_timestamp = calculate_next_15m_market_timestamp();
    let target_slug = format!("btc-updown-15m-{}", next_timestamp);
    
    println!("🎯 Target market: {}", target_slug);
    println!("📅 Timestamp: {}", next_timestamp);
    println!("🔌 Connecting to CLOB WebSocket...\n");
    
    // Connect to WebSocket
    let (ws_stream, _) = connect_async(CLOB_WSS_URL).await?;
    println!("✅ Connected to CLOB WebSocket\n");
    
    let (mut write, mut read) = ws_stream.split();
    
    // First, we need to get the market token IDs for the target market
    // We'll fetch this from Gamma API first
    println!("📡 Fetching market token IDs from Gamma API...");
    let token_ids = get_market_token_ids(&target_slug).await?;
    
    if token_ids.is_empty() {
        println!("⏳ Market not available yet. Waiting and retrying...");
        return Ok(());
    }
    
    println!("✅ Found token IDs: {:?}\n", token_ids);
    
    // Subscribe to market channel for these tokens
    for token_id in &token_ids {
        let subscribe_msg = json!({
            "type": "subscribe",
            "channel": "market",
            "market": token_id
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        println!("📬 Subscribed to token: {}", token_id);
    }
    
    println!("\n🎧 Listening for real-time market updates...\n");
    println!("{}", "=".repeat(80));
    
    // Listen for messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    print_market_update(&data, &target_slug);
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 WebSocket connection closed");
                break;
            }
            Err(e) => {
                println!("❌ WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// Get market token IDs from Gamma API
async fn get_market_token_ids(slug: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = "https://gamma-api.polymarket.com/markets";
    let client = reqwest::Client::new();
    
    let response = client
        .get(url)
        .query(&[("slug", slug)])
        .send()
        .await?;
    
    let markets: Vec<Value> = response.json().await?;
    
    if markets.is_empty() {
        return Ok(Vec::new());
    }
    
    // Extract clobTokenIds
    if let Some(clob_token_ids) = markets[0].get("clobTokenIds").and_then(|v| v.as_str()) {
        // Parse the comma-separated token IDs
        let tokens: Vec<String> = clob_token_ids
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        return Ok(tokens);
    }
    
    Ok(Vec::new())
}

/// Print market update from WebSocket
fn print_market_update(data: &Value, _target_slug: &str) {
    println!("📊 Market Update Received:");
    
    // Print the message type
    if let Some(msg_type) = data.get("type").and_then(|t| t.as_str()) {
        println!("   Type: {}", msg_type);
    }
    
    // Print market info
    if let Some(market) = data.get("market") {
        println!("   Market: {}", market);
    }
    
    // Print asset_id (token ID)
    if let Some(asset_id) = data.get("asset_id").and_then(|a| a.as_str()) {
        println!("   Token ID: {}", asset_id);
    }
    
    // Print price information if available
    if let Some(price) = data.get("price") {
        println!("   Price: {}", price);
    }
    
    if let Some(last_price) = data.get("last") {
        println!("   Last Price: {}", last_price);
    }
    
    if let Some(bid) = data.get("bid") {
        println!("   Bid: {}", bid);
    }
    
    if let Some(ask) = data.get("ask") {
        println!("   Ask: {}", ask);
    }
    
    println!("{}", "-".repeat(80));
}

/// Calculate the Unix timestamp for the NEXT 15-minute market interval
fn calculate_next_15m_market_timestamp() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    
    let interval_seconds = 900; // 15 minutes
    let current_interval_start = (now / interval_seconds) * interval_seconds;
    
    // NEXT interval start time
    current_interval_start + interval_seconds
}