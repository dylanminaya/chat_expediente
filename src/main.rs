use anyhow::Result;

mod claude;
mod web_server;

use claude::ClaudeClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    let region = "us-east-1";
    let port = 3000;
    
    println!("🚀 Initializing connection to Claude Haiku 3.5...");
    println!("🌍 Region: {}", region);
    
    let claude = ClaudeClient::new(region).await?;
    println!("✅ Connected successfully!");
    
    println!("🌐 Starting web server on port {}...", port);
    web_server::create_web_server(claude, port).await?;
    
    Ok(())
}
