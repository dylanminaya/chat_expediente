use anyhow::Result;
use clap::Parser;

mod cli;
mod claude;
mod document;
mod chat;
mod web_server;

use cli::Args;
use claude::ClaudeClient;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🚀 Initializing connection to Claude Haiku 3.5...");
    println!("🌍 Region: {}", args.region);
    
    let claude = ClaudeClient::new(&args.region).await?;
    println!("✅ Connected successfully!");
    
    if args.web {
        println!("🌐 Starting web server mode...");
        web_server::create_web_server(claude, args.port).await?;
    } else {
        // Default to interactive mode
        chat::interactive_chat(&claude).await?;
    }
    
    Ok(())
}
