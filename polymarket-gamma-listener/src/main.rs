mod gamma_check_btc_15m;
mod clob_check_btc_15m;

use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("🚀 POLYMARKET BTC 15-MINUTE MONITOR");
    println!("{}", "=".repeat(80));
    println!("📊 Gamma: Monitors CURRENT market (sleeps between intervals)");
    println!("⚡ CLOB:  Monitors NEXT market (real-time WebSocket)");
    println!("{}", "=".repeat(80));
    println!();
    
    // Spawn both tasks concurrently
    let gamma_task = tokio::spawn(async {
        run_gamma_monitor().await
    });
    
    let clob_task = tokio::spawn(async {
        run_clob_monitor().await
    });
    
    // Wait for both tasks (they run forever)
    let _ = tokio::try_join!(gamma_task, clob_task);
    
    Ok(())
}

/// Run Gamma monitor loop
async fn run_gamma_monitor() {
    use std::time::Duration;
    use tokio::time::sleep;
    
    loop {
        println!("\n{}", "=".repeat(80));
        println!("⏰ GAMMA: Checking for CURRENT market...");
        println!("{}", "=".repeat(80));
        
        match gamma_check_btc_15m::get_current_market().await {
            Ok(Some(market)) => {
                gamma_check_btc_15m::print_market_details(&market);
            }
            Ok(None) => {
                println!("⚠️  GAMMA: No active market found");
            }
            Err(e) => {
                println!("❌ GAMMA Error: {}", e);
            }
        }
        
        let sleep_seconds = gamma_check_btc_15m::seconds_until_next_interval();
        let minutes = sleep_seconds / 60;
        let seconds = sleep_seconds % 60;
        
        println!("\n💤 GAMMA: Sleeping for {}m {}s until next interval...\n", minutes, seconds);
        sleep(Duration::from_secs(sleep_seconds)).await;
    }
}

/// Run CLOB monitor loop
async fn run_clob_monitor() {
    loop {
        let result = clob_check_btc_15m::monitor_next_market_websocket().await;
        
        match result {
            Ok(_) => {
                println!("✅ CLOB: Monitor cycle completed");
            }
            Err(e) => {
                // Convert error to string immediately to drop the Box<dyn Error>
                let error_string = format!("{}", e);
                drop(e); // Explicitly drop the error
                
                println!("❌ CLOB Error: {}", error_string);
                println!("🔄 CLOB: Retrying in 10 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        }
    }
}