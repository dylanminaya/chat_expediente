use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use anyhow::Result;
use regex::Regex;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

use crate::claude::{ClaudeClient, Message, MessageContent};

#[derive(Clone)]
pub struct AppState {
    claude: Arc<ClaudeClient>,
    conversations: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    global_context: Arc<String>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    message: String,
    conversation_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocumentReference {
    document_id: String,
    version_id: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    response: String,
    conversation_id: String,
    input_tokens: u32,
    output_tokens: u32,
    document_references: Vec<DocumentReference>,
}

#[derive(Deserialize)]
pub struct ConversationQuery {
    id: Option<String>,
}

#[derive(Deserialize)]
pub struct DocumentQuery {
    document_id: String,
    version_id: String,
    conversation_id: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentResponse {
    success: bool,
    document_id: String,
    version_id: String,
    content: Option<String>,
    binaries: Option<Vec<String>>,
    error: Option<String>,
    // claude_response: String,
    conversation_id: String,
}

pub async fn create_web_server(claude: ClaudeClient, port: u16) -> Result<()> {
    // Load global context
    let global_context = match ClaudeClient::load_global_context() {
        Ok(context) => {
            println!("✅ Global context loaded successfully");
            context
        }
        Err(e) => {
            println!("⚠️  Warning: Failed to load global context: {}", e);
            "You are an AI assistant specialized in analyzing financial and legal documents.".to_string()
        }
    };

    let state = AppState {
        claude: Arc::new(claude),
        conversations: Arc::new(Mutex::new(HashMap::new())),
        global_context: Arc::new(global_context),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/chat", post(handle_chat))
        .route("/api/conversations", get(get_conversations))
        .route("/api/document", get(fetch_document))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🌐 Web server running on http://localhost:{}", port);
    println!("📱 Open your browser and navigate to the URL above");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

fn extract_document_references(text: &str) -> Vec<DocumentReference> {
    let mut references = Vec::new();
    
    // Pattern 1: Bracketed format like [Document ID: xxx, Document Version ID: yyy]
    // Support English, Spanish, and French
    let bracket_re = Regex::new(r"(?i)\[(?:.*?)?(?:document\s+id|documentid|documento\s+id|documentoid|id\s+du\s+document|iddudocument):\s*([a-f0-9]{24})(?:.*?)(?:document\s+version\s+id|documentversionid|version\s+id|versionid|documento\s+version\s+id|documentoversionid|versión\s+id|versionid|id\s+de\s+version\s+du\s+document|iddeversiondudocument|id\s+de\s+version|iddeversion):\s*([a-f0-9]{24})(?:.*?)?\]").unwrap();
    
    for cap in bracket_re.captures_iter(text) {
        let document_id = cap[1].to_string();
        let version_id = cap[2].to_string();
        
        references.push(DocumentReference {
            document_id,
            version_id,
        });
    }
    
    // If we found bracketed references, return them
    if !references.is_empty() {
        return references;
    }
    
    // Pattern 2: Line-by-line format
    // Document ID: xxx / Documento ID: xxx / ID du document: xxx
    // Document Version ID: yyy / Documento Version ID: yyy / ID de version du document: yyy
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let doc_id_re = Regex::new(r"(?i)(?:document\s+id|documentid|documento\s+id|documentoid|id\s+du\s+document|iddudocument):\s*([a-f0-9]{24})").unwrap();
        let version_id_re = Regex::new(r"(?i)(?:document\s+version\s+id|documentversionid|version\s+id|versionid|documento\s+version\s+id|documentoversionid|versión\s+id|versionid|id\s+de\s+version\s+du\s+document|iddeversiondudocument|id\s+de\s+version|iddeversion):\s*([a-f0-9]{24})").unwrap();
        
        if let Some(doc_cap) = doc_id_re.captures(lines[i]) {
            let document_id = doc_cap[1].to_string();
            let mut version_id = document_id.clone(); // Default fallback
            
            // Look for version ID in the next few lines
            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                if let Some(ver_cap) = version_id_re.captures(lines[j]) {
                    version_id = ver_cap[1].to_string();
                    break;
                }
            }
            
            references.push(DocumentReference {
                document_id,
                version_id,
            });
        }
        i += 1;
    }
    
    // If we found line-by-line references, return them
    if !references.is_empty() {
        return references;
    }
    
    // Pattern 3: Same-line format with various separators
    // Support English, Spanish, and French
    let same_line_re = Regex::new(r"(?i)(?:document\s+id|documentid|documento\s+id|documentoid|id\s+du\s+document|iddudocument):\s*([a-f0-9]{24})(?:\s*[-,\s]+\s*(?:document\s+version\s+id|documentversionid|version\s+id|versionid|documento\s+version\s+id|documentoversionid|versión\s+id|versionid|id\s+de\s+version\s+du\s+document|iddeversiondudocument|id\s+de\s+version|iddeversion):\s*([a-f0-9]{24}))").unwrap();
    
    for cap in same_line_re.captures_iter(text) {
        let document_id = cap[1].to_string();
        let version_id = cap[2].to_string();
        
        references.push(DocumentReference {
            document_id,
            version_id,
        });
    }
    
    // If we found same-line references, return them
    if !references.is_empty() {
        return references;
    }
    
    // Pattern 4: Fallback - collect all Document IDs and Version IDs separately
    // Support English, Spanish, and French
    let doc_id_re = Regex::new(r"(?i)(?:document\s+id|documentid|documento\s+id|documentoid|id\s+du\s+document|iddudocument):\s*([a-f0-9]{24})").unwrap();
    let version_id_re = Regex::new(r"(?i)(?:document\s+version\s+id|documentversionid|version\s+id|versionid|documento\s+version\s+id|documentoversionid|versión\s+id|versionid|id\s+de\s+version\s+du\s+document|iddeversiondudocument|id\s+de\s+version|iddeversion):\s*([a-f0-9]{24})").unwrap();
    
    let document_ids: Vec<String> = doc_id_re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect();
    
    let version_ids: Vec<String> = version_id_re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect();
    
    // Match document IDs with version IDs by position
    for (i, document_id) in document_ids.iter().enumerate() {
        let version_id = if i < version_ids.len() {
            version_ids[i].clone()
        } else {
            // If no corresponding version ID, use the document ID as fallback
            document_id.clone()
        };
        
        references.push(DocumentReference {
            document_id: document_id.clone(),
            version_id,
        });
    }
    
    references
}

fn extract_pdf_text(base64_data: &str) -> Result<String, String> {
    // Remove the data URL prefix if present
    let clean_base64 = if base64_data.starts_with("data:application/pdf;base64,") {
        &base64_data[28..] // Remove "data:application/pdf;base64,"
    } else {
        base64_data
    };
    
    // Decode base64 to bytes
    let pdf_bytes = general_purpose::STANDARD
        .decode(clean_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Extract text from PDF
    match pdf_extract::extract_text_from_mem(&pdf_bytes) {
        Ok(text) => {
            if text.trim().is_empty() {
                Ok("📄 PDF document processed but no extractable text content found. This may be a scanned document or contain only images.".to_string())
            } else {
                Ok(text)
            }
        }
        Err(e) => Err(format!("Failed to extract text from PDF: {}", e))
    }
}

async fn handle_chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let conversation_id = request.conversation_id.unwrap_or_else(|| {
        Uuid::new_v4().to_string()
    });

    let mut conversations = state.conversations.lock().await;
    let conversation = conversations.entry(conversation_id.clone()).or_insert_with(Vec::new);

    let message = Message {
        role: "user".to_string(),
        content: MessageContent::Text(request.message),
    };

    conversation.push(message);
    
    // Use global context as system message for all conversations
    let system_context = Some(state.global_context.as_ref().clone());

    // Send to Claude with system context
    match state.claude.send_message_with_system(conversation, system_context).await {
        Ok(response) => {
            // Add Claude's response to conversation
            conversation.push(Message {
                role: "assistant".to_string(),
                content: MessageContent::Text(response.clone()),
            });

            let document_references = extract_document_references(&response);

            Ok(Json(ChatResponse {
                response,
                conversation_id,
                input_tokens: 0, // You'd need to modify ClaudeClient to return these
                output_tokens: 0,
                document_references,
            }))
        }
        Err(_) => {
            conversation.pop(); // Remove the failed message
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn fetch_document(
    State(state): State<AppState>,
    Query(params): Query<DocumentQuery>,
) -> Json<DocumentResponse> {
    // Get document endpoint from environment variable or use default
    let base_url = std::env::var("DOCUMENT_API_BASE_URL")
        .unwrap_or_else(|_| "https://cloudx.prodoctivity.com/api/app".to_string());
    
    // Get authentication token from environment
    let token = std::env::var("PRODOCTIVITY_TOKEN")
        .unwrap_or_else(|_| String::new());
    
    let document_endpoint = format!(
        "{}/documents/{}/versions/{}",
        base_url, params.document_id, params.version_id
    );
    
    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    // Prepare request with authentication header
    let mut request_builder = client.get(&document_endpoint);
    
    if !token.is_empty() {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
    }
    
    // Make HTTP request to fetch the document
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            
            if status.is_success() {
                match response.text().await {
                    Ok(content) => {
                        // Try to parse as JSON to extract the actual document structure
                        let (document_content, binaries) = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Extract binaries field (could be null or array)
                            let mut binaries = json_value.get("binaries")
                                .and_then(|b| {
                                    if b.is_null() {
                                        None
                                    } else {
                                        b.as_array().map(|arr| arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect::<Vec<String>>())
                                    }
                                });
                            
                            // If binaries is null, check for base64 data in nested document.data field
                            if binaries.is_none() {
                                if let Some(document) = json_value.get("document") {
                                    // Check document.data field for base64 content
                                    if let Some(data_str) = document.get("data").and_then(|d| d.as_str()) {
                                        if data_str.starts_with("application/pdf;base64,") || data_str.starts_with("data:application/pdf;base64,") {
                                            let base64_data = if data_str.starts_with("data:") {
                                                data_str.to_string()
                                            } else {
                                                format!("data:{}", data_str)
                                            };
                                            binaries = Some(vec![base64_data]);
                                        }
                                    }
                                    
                                    // Also check document.binaries field
                                    if binaries.is_none() {
                                        if let Some(doc_binaries) = document.get("binaries").and_then(|b| b.as_array()) {
                                            let extracted_binaries: Vec<String> = doc_binaries.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect();
                                            if !extracted_binaries.is_empty() {
                                                binaries = Some(extracted_binaries);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Extract the actual document content if it exists
                            let document_content = json_value.get("data")
                                .and_then(|d| d.as_object())
                                .map(|obj| serde_json::to_string_pretty(obj).unwrap_or_else(|_| "Failed to serialize data".to_string()))
                                .or_else(|| {
                                    // Fallback to the entire JSON if no data field
                                    Some(serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| content.clone()))
                                });
                            
                            (document_content, binaries)
                        } else {
                            // Not JSON, treat as plain text
                            (Some(content), None)
                        };

                        // Always send the document content to Claude and add it to conversation context
                        let conversation_id = params.conversation_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
                        
                        // Extract text content from PDF if available
                        let mut text_content = String::new();
                        
                        if let Some(ref binaries) = binaries {
                            for binary_data in binaries {
                                if binary_data.starts_with("data:application/pdf;base64,") {
                                    match extract_pdf_text(binary_data) {
                                        Ok(pdf_text) => {
                                            text_content.push_str("📄 PDF Content:\n");
                                            // Truncate PDF content if too long (keep first 50,000 characters)
                                            let truncated_text = if pdf_text.len() > 50000 {
                                                format!("{}...\n\n[Content truncated - PDF is very long. Showing first 50,000 characters]", &pdf_text[..50000])
                                            } else {
                                                pdf_text
                                            };
                                            text_content.push_str(&truncated_text);
                                            text_content.push_str("\n\n");
                                        }
                                        Err(e) => {
                                            text_content.push_str(&format!("📄 PDF document detected but failed to extract text: {}\n\n", e));
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Add document metadata if no PDF content was extracted
                        if text_content.is_empty() {
                            if let Some(ref doc_content) = document_content {
                                // Truncate metadata if too long
                                let truncated_metadata = if doc_content.len() > 10000 {
                                    format!("{}...\n\n[Metadata truncated]", &doc_content[..10000])
                                } else {
                                    doc_content.clone()
                                };
                                text_content.push_str(&format!("📋 Document Metadata:\n{}\n\n", truncated_metadata));
                            }
                        } else {
                            // Add metadata after PDF content (truncated)
                            if let Some(ref doc_content) = document_content {
                                let truncated_metadata = if doc_content.len() > 5000 {
                                    format!("{}...\n\n[Metadata truncated]", &doc_content[..5000])
                                } else {
                                    doc_content.clone()
                                };
                                text_content.push_str(&format!("📋 Document Metadata:\n{}\n\n", truncated_metadata));
                            }
                        }
                        
                        let message_text = format!(
                            "Document fetched and added to context:\n\n\
                            Document ID: {}\n\
                            Document Version ID: {}\n\n\
                            {}",
                            params.document_id,
                            params.version_id,
                            text_content
                        );

                        // Final safety check - if message is still too long, truncate further
                        let final_message_text = if message_text.len() > 100000 {
                            let truncated = &message_text[..100000];
                            format!("{}...\n\n[Message truncated to prevent token limit exceeded]", truncated)
                        } else {
                            message_text
                        };

                        println!("🔍 Final message text length: {} characters", final_message_text.len());
                        println!("🔍 Text content preview: {}", &text_content[..std::cmp::min(500, text_content.len())]);

                        // Get or create conversation
                        let mut conversations = state.conversations.lock().await;
                        let conversation = conversations.entry(conversation_id.clone()).or_insert_with(Vec::new);

                        println!("🔍 Current conversation length: {} messages", conversation.len());

                        // Limit conversation history to prevent token overflow
                        // Keep only the last 10 messages (5 exchanges) to stay within token limits
                        if conversation.len() > 10 {
                            let keep_from = conversation.len() - 10;
                            conversation.drain(0..keep_from);
                            println!("🔄 Trimmed conversation history, now {} messages", conversation.len());
                        }

                        let message = Message {
                            role: "user".to_string(),
                            content: MessageContent::Text(final_message_text),
                        };

                        // Add the document content to conversation context without sending to Claude
                        conversation.push(message);

                        // Create a simple confirmation response without calling Claude
                        // let claude_response = format!(
                        //     "✅ Document added to conversation context:\n\n\
                        //     📄 **Document ID:** {}\n\
                        //     📄 **Document Version ID:** {}\n\n\
                        //     The document content is now available in our conversation. You can ask me questions about this document and I'll analyze it for you.",
                        //     params.document_id,
                        //     params.version_id
                        // );

                        // // Add the confirmation as an assistant message
                        // conversation.push(Message {
                        //     role: "assistant".to_string(),
                        //     content: MessageContent::Text(claude_response.clone()),
                        // });

                        Json(DocumentResponse {
                            success: true,
                            document_id: params.document_id,
                            version_id: params.version_id,
                            content: document_content,
                            binaries,
                            error: None,
                            // claude_response,
                            conversation_id,
                        })
                    }
                    Err(e) => {
                        Json(DocumentResponse {
                            success: false,
                            document_id: params.document_id,
                            version_id: params.version_id,
                            content: None,
                            binaries: None,
                            error: Some(format!("Failed to read document content: {}", e)),
                            // claude_response: String::new(),
                            conversation_id: params.conversation_id.unwrap_or_default(),
                        })
                    }
                }
            } else {
                let error_message = if status == 401 {
                    "Authentication failed - please check your PRODOCTIVITY_TOKEN".to_string()
                } else if status == 403 {
                    "Access forbidden - insufficient permissions".to_string()
                } else if status == 404 {
                    "Document not found".to_string()
                } else {
                    format!("HTTP error: {} - {}", status, status.canonical_reason().unwrap_or("Unknown error"))
                };
                
                Json(DocumentResponse {
                    success: false,
                    document_id: params.document_id,
                    version_id: params.version_id,
                    content: None,
                    binaries: None,
                    error: Some(error_message),
                    // claude_response: String::new(),
                    conversation_id: params.conversation_id.unwrap_or_default(),
                })
            }
        }
        Err(e) => {
            Json(DocumentResponse {
                success: false,
                document_id: params.document_id,
                version_id: params.version_id,
                content: None,
                binaries: None,
                error: Some(format!("Network error: {}", e)),
                // claude_response: String::new(),
                conversation_id: params.conversation_id.unwrap_or_default(),
            })
        }
    }
}

async fn get_conversations(
    State(state): State<AppState>,
    Query(params): Query<ConversationQuery>,
) -> Json<Vec<Message>> {
    let conversations = state.conversations.lock().await;
    
    if let Some(id) = params.id {
        if let Some(conversation) = conversations.get(&id) {
            Json(conversation.clone())
        } else {
            Json(vec![])
        }
    } else {
        Json(vec![])
    }
} 