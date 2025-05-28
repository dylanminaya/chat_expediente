use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use std::io::Read;
use dotext::*;
use crate::claude::Message;

pub fn load_document_as_text(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(anyhow!("File not found: {}", file_path));
    }
    
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "txt" | "md" | "json" | "xml" | "csv" | "html" | "htm" => {
            let content = fs::read_to_string(file_path)?;
            Ok(content)
        }
        "docx" => {
            // Extract text from DOCX file
            match extract_docx_text(file_path) {
                Ok(text) => Ok(text),
                Err(e) => {
                    // Fallback to binary file info if DOCX parsing fails
                    let content = fs::read(file_path)?;
                    let file_name = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown");
                    
                    Ok(format!(
                        "[DOCX file: {} ({} bytes) - Text extraction failed: {}]",
                        file_name,
                        content.len(),
                        e
                    ))
                }
            }
        }
        _ => {
            // For other binary files, we'll read them as bytes and provide a summary
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

fn extract_docx_text(file_path: &str) -> Result<String> {
    let mut file = Docx::open(file_path)?;
    let mut text_content = String::new();
    file.read_to_string(&mut text_content)?;
    
    if text_content.trim().is_empty() {
        return Err(anyhow!("No text content found in DOCX file"));
    }
    
    Ok(text_content)
}

pub fn create_message_with_files(text: &str, file_paths: &[String]) -> Result<Message> {
    if file_paths.is_empty() {
        // Simple text message
        return Ok(Message {
            role: "user".to_string(),
            content: crate::claude::MessageContent::Text(text.to_string()),
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
        content: crate::claude::MessageContent::Text(combined_content),
    })
} 