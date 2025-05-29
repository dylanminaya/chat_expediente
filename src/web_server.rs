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

#[derive(Serialize)]
pub struct ChatResponse {
    response: String,
    conversation_id: String,
    input_tokens: u32,
    output_tokens: u32,
    document_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConversationQuery {
    id: Option<String>,
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

fn extract_document_ids(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?i)(?:document\s+id|documentid):\s*([a-f0-9]{24})").unwrap();
    re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

async fn handle_chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let conversation_id = request.conversation_id.unwrap_or_else(|| {
        uuid::Uuid::new_v4().to_string()
    });

    let mut conversations = state.conversations.lock().await;
    let conversation = conversations.entry(conversation_id.clone()).or_insert_with(Vec::new);

    // Create simple text message
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

            let document_ids = extract_document_ids(&response);

            Ok(Json(ChatResponse {
                response,
                conversation_id,
                input_tokens: 0, // You'd need to modify ClaudeClient to return these
                output_tokens: 0,
                document_ids,
            }))
        }
        Err(_) => {
            conversation.pop(); // Remove the failed message
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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