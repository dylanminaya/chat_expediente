use clap::Parser;

#[derive(Parser)]
#[command(name = "chat_expediente")]
#[command(about = "Interactive chat with Claude Haiku 3.5 via AWS Bedrock")]
pub struct Args {
    #[arg(short, long, default_value = "us-east-1")]
    pub region: String,
    
    #[arg(short, long)]
    pub web: bool,
    
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
} 