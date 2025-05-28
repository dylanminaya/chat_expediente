use anyhow::Result;
use std::io::{self, Write};
use crate::claude::{ClaudeClient, Message};
use crate::document::create_message_with_files;

pub async fn interactive_chat(claude: &ClaudeClient) -> Result<()> {
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
                            crate::claude::MessageContent::Text(text) => text.clone(),
                            crate::claude::MessageContent::Multimodal(blocks) => {
                                let text_parts: Vec<String> = blocks.iter()
                                    .filter_map(|block| match block {
                                        crate::claude::ContentBlock::Text { text } => Some(text.clone()),
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
            if std::path::Path::new(file_path).exists() {
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
                if std::path::Path::new(file_path).exists() {
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
                    content: crate::claude::MessageContent::Text(response),
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