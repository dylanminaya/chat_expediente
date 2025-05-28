use axum::{
    extract::{Multipart, Query, State},
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

use crate::claude::{ClaudeClient, Message, MessageContent};
use crate::document::create_message_with_files;

#[derive(Clone)]
pub struct AppState {
    claude: Arc<ClaudeClient>,
    conversations: Arc<Mutex<HashMap<String, Vec<Message>>>>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    message: String,
    conversation_id: Option<String>,
    files: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    response: String,
    conversation_id: String,
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
pub struct ConversationQuery {
    id: Option<String>,
}

pub async fn create_web_server(claude: ClaudeClient, port: u16) -> Result<()> {
    let state = AppState {
        claude: Arc::new(claude),
        conversations: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/chat", post(handle_chat))
        .route("/api/conversations", get(get_conversations))
        .route("/api/upload", post(handle_file_upload))
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

async fn handle_chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let conversation_id = request.conversation_id.unwrap_or_else(|| {
        uuid::Uuid::new_v4().to_string()
    });

    let mut conversations = state.conversations.lock().await;
    let conversation = conversations.entry(conversation_id.clone()).or_insert_with(Vec::new);

    // Create message with files if provided
    let file_paths = request.files.unwrap_or_default();
    let message = match create_message_with_files(&request.message, &file_paths) {
        Ok(msg) => msg,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    conversation.push(message);

    // Send to Claude
    match state.claude.send_message(conversation).await {
        Ok(response) => {
            // Add Claude's response to conversation
            conversation.push(Message {
                role: "assistant".to_string(),
                content: MessageContent::Text(response.clone()),
            });

            Ok(Json(ChatResponse {
                response,
                conversation_id,
                input_tokens: 0, // You'd need to modify ClaudeClient to return these
                output_tokens: 0,
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

async fn handle_file_upload(
    mut multipart: Multipart,
) -> Result<Json<Vec<String>>, StatusCode> {
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            // Save file to uploads directory
            let file_path = format!("uploads/{}", file_name);
            tokio::fs::create_dir_all("uploads").await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            tokio::fs::write(&file_path, data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            uploaded_files.push(file_path);
        }
    }
    
    Ok(Json(uploaded_files))
} 