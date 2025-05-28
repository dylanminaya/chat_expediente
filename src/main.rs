use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{Client, primitives::Blob};
use aws_types::region::Region;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use clap::Parser;

mod web_server;

#[derive(Parser)]
#[command(name = "chat_expediente")]
#[command(about = "Chat with Claude Haiku 3.5 via AWS Bedrock")]
struct Args {
    #[arg(short, long, default_value = "us-east-1")]
    region: String,
    
    #[arg(short, long)]
    message: Option<String>,
    
    #[arg(short, long, default_value = "false")]
    interactive: bool,
    
    #[arg(short, long)]
    files: Vec<String>,
    
    #[arg(short, long)]
    web: bool,
    
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[derive(Serialize)]
struct ClaudeRequest {
    anthropic_version: String,
    max_tokens: u32,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: MessageContent,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ResponseContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

struct ClaudeClient {
    client: Client,
    model_id: String,
}

impl ClaudeClient {
    async fn new(region: &str) -> Result<Self> {
        let region = Region::new(region.to_string());
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .load()
            .await;
        
        let client = Client::new(&config);
        let model_id = "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string();
        
        Ok(ClaudeClient { client, model_id })
    }
    
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        let request = ClaudeRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: 4096,
            messages: messages.to_vec(),
            temperature: 0.7,
        };
        
        let body = serde_json::to_string(&request)?;
        let blob = Blob::new(body.as_bytes());
        
        let response = self.client
            .invoke_model()
            .model_id(&self.model_id)
            .content_type("application/json")
            .body(blob)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to invoke model: {}", e))?;
        
        let response_body = response.body().as_ref();
        let claude_response: ClaudeResponse = serde_json::from_slice(response_body)
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        
        if let Some(content) = claude_response.content.first() {
            println!("📊 Tokens used - Input: {}, Output: {}", 
                     claude_response.usage.input_tokens, 
                     claude_response.usage.output_tokens);
            Ok(content.text.clone())
        } else {
            Err(anyhow!("No content in response"))
        }
    }
}

fn load_document_as_text(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(anyhow!("File not found: {}", file_path));
    }
    
    // For now, we'll only support text-based files
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "txt" | "md" | "json" | "xml" | "csv" | "html" | "htm" => {
            let content = fs::read_to_string(file_path)?;
            Ok(content)
        }
        _ => {
            // For binary files, we'll read them as bytes and provide a summary
            let content = fs::read(file_path)?;
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            Ok(format!(
                "[Binary file: {} ({} bytes, type: {})]",
                file_name,
                content.len(),
                extension
            ))
        }
    }
}

fn create_message_with_files(text: &str, file_paths: &[String]) -> Result<Message> {
    if file_paths.is_empty() {
        // Simple text message
        return Ok(Message {
            role: "user".to_string(),
            content: MessageContent::Text(text.to_string()),
        });
    }
    
    // Multimodal message with text and document contents
    let mut content_parts = vec![format!("User question: {}", text)];
    
    for file_path in file_paths {
        match load_document_as_text(file_path) {
            Ok(doc_content) => {
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");
                
                println!("📎 Loaded document: {}", file_path);
                content_parts.push(format!(
                    "\n--- Document: {} ---\n{}\n--- End of Document ---\n",
                    file_name,
                    doc_content
                ));
            }
            Err(e) => {
                println!("⚠️  Failed to load {}: {}", file_path, e);
            }
        }
    }
    
    let combined_content = content_parts.join("\n");
    
    Ok(Message {
        role: "user".to_string(),
        content: MessageContent::Text(combined_content),
    })
}

async fn interactive_chat(claude: &ClaudeClient) -> Result<()> {
    let mut conversation: Vec<Message> = Vec::new();
    
    println!("🤖 Claude Haiku 3.5 Chat");
    println!("Type 'quit', 'exit', or 'bye' to end the conversation");
    println!("Type 'clear' to clear the conversation history");
    println!("Type 'history' to see the conversation history");
    println!("Type 'file <path>' to attach a document to your next message");
    println!("Type 'files <path1> <path2> ...' to attach multiple documents");
    println!("{}", "=".repeat(50));
    
    let mut pending_files: Vec<String> = Vec::new();
    
    loop {
        if !pending_files.is_empty() {
            println!("📎 Pending files: {}", pending_files.join(", "));
        }
        print!("\n💬 You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input.to_lowercase().as_str() {
            "quit" | "exit" | "bye" => {
                println!("👋 Goodbye!");
                break;
            }
            "clear" => {
                conversation.clear();
                pending_files.clear();
                println!("🧹 Conversation history and pending files cleared!");
                continue;
            }
            "history" => {
                if conversation.is_empty() {
                    println!("📝 No conversation history yet.");
                } else {
                    println!("📝 Conversation History:");
                    for (i, msg) in conversation.iter().enumerate() {
                        let role_emoji = if msg.role == "user" { "💬" } else { "🤖" };
                        let content_preview = match &msg.content {
                            MessageContent::Text(text) => text.clone(),
                            MessageContent::Multimodal(blocks) => {
                                let text_parts: Vec<String> = blocks.iter()
                                    .filter_map(|block| match block {
                                        ContentBlock::Text { text } => Some(text.clone()),
                                    })
                                    .collect();
                                text_parts.join(" ")
                            }
                        };
                        println!("{}. {} {}: {}", i + 1, role_emoji, msg.role, content_preview);
                    }
                }
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        // Handle file commands
        if input.starts_with("file ") {
            let file_path = input.strip_prefix("file ").unwrap().trim();
            if Path::new(file_path).exists() {
                pending_files.push(file_path.to_string());
                println!("📎 Added file: {}", file_path);
            } else {
                println!("❌ File not found: {}", file_path);
            }
            continue;
        }
        
        if input.starts_with("files ") {
            let file_paths: Vec<&str> = input.strip_prefix("files ").unwrap().split_whitespace().collect();
            for file_path in file_paths {
                if Path::new(file_path).exists() {
                    pending_files.push(file_path.to_string());
                    println!("📎 Added file: {}", file_path);
                } else {
                    println!("❌ File not found: {}", file_path);
                }
            }
            continue;
        }
        
        // Create message with text and any pending files
        let message = create_message_with_files(input, &pending_files)?;
        conversation.push(message);
        pending_files.clear(); // Clear files after using them
        
        print!("🤖 Claude: ");
        io::stdout().flush()?;
        
        match claude.send_message(&conversation).await {
            Ok(response) => {
                println!("{}", response);
                
                // Add Claude's response to conversation
                conversation.push(Message {
                    role: "assistant".to_string(),
                    content: MessageContent::Text(response),
                });
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                // Remove the user message if there was an error
                conversation.pop();
            }
        }
    }
    
    Ok(())
}

async fn single_message(claude: &ClaudeClient, message: &str, file_paths: &[String]) -> Result<()> {
    let message_obj = create_message_with_files(message, file_paths)?;
    let messages = vec![message_obj];
    
    println!("💬 You: {}", message);
    if !file_paths.is_empty() {
        println!("📎 With files: {}", file_paths.join(", "));
    }
    print!("🤖 Claude: ");
    io::stdout().flush()?;
    
    match claude.send_message(&messages).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}

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
    } else if let Some(message) = args.message {
        single_message(&claude, &message, &args.files).await?;
    } else if args.interactive {
        interactive_chat(&claude).await?;
    } else {
        // Default to interactive mode
        interactive_chat(&claude).await?;
    }
    
    Ok(())
}
