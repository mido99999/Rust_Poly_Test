use reqwest;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current active BTC 15-minute market
pub async fn get_current_market() -> Result<Option<Value>, Box<dyn std::error::Error>> {
    // Calculate the timestamp for current 15m market
    let market_timestamp = calculate_current_15m_market_timestamp();
    let slug = format!("btc-updown-15m-{}", market_timestamp);
    
    println!("🔍 Searching for market: {}", slug);
    println!("📅 Timestamp: {}\n", market_timestamp);

    // Search for this specific market
    let url = "https://gamma-api.polymarket.com/markets";
    let client = reqwest::Client::new();
    
    let response = client
        .get(url)
        .query(&[
            ("closed", "false"),
            ("slug", slug.as_str())
        ])
        .send()
        .await?;

    let markets: Vec<Value> = response.json().await?;

    if markets.is_empty() {
        println!("❌ Market not found");
        Ok(None)
    } else {
        println!("✅ Market found!\n");
        Ok(Some(markets[0].clone()))
    }
}

/// Print market details in a formatted way
pub fn print_market_details(market: &Value) {
    println!("{}", "=".repeat(80));
    println!("📊 MARKET DETAILS");
    println!("{}", "=".repeat(80));
    
    if let Some(question) = market.get("question").and_then(|q| q.as_str()) {
        println!("📋 Question: {}", question);
    }
    
    if let Some(slug) = market.get("slug").and_then(|s| s.as_str()) {
        println!("🔗 Slug: {}", slug);
    }
    
    if let Some(end_date) = market.get("endDate").and_then(|d| d.as_str()) {
        println!("⏰ End Date: {}", end_date);
    }
    
    if let Some(active) = market.get("active") {
        println!("🟢 Active: {}", active);
    }
    
    if let Some(volume) = market.get("volume24hr") {
        println!("💰 24h Volume: {}", volume);
    }
    
    println!("{}", "=".repeat(80));
}

/// Calculate the Unix timestamp for the current 15-minute market interval
fn calculate_current_15m_market_timestamp() -> i64 {
    // Get current Unix timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    
    // Round down to nearest 15-minute interval (900 seconds = 15 minutes)
    let interval_seconds = 900;
    let current_interval_start = (now / interval_seconds) * interval_seconds;
    
    // The timestamp in the slug is the START time of the current interval
    current_interval_start
}

/// Calculate seconds until the next 15-minute interval (00, 15, 30, 45)
pub fn seconds_until_next_interval() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    let interval_seconds = 900; // 15 minutes
    let current_interval_start = (now / interval_seconds) * interval_seconds;
    let next_interval_start = current_interval_start + interval_seconds;
    
    (next_interval_start - now) as u64
}