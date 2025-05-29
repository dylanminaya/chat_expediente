use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{Client, primitives::Blob};
use aws_types::region::Region;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
pub struct ClaudeRequest {
    anthropic_version: String,
    max_tokens: u32,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ContentBlock {
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

pub struct ClaudeClient {
    client: Client,
    model_id: String,
}

impl ClaudeClient {
    pub async fn new(region: &str) -> Result<Self> {
        let region = Region::new(region.to_string());
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .load()
            .await;
        
        let client = Client::new(&config);
        let model_id = "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string();
        
        Ok(ClaudeClient { client, model_id })
    }
    
    pub fn load_global_context() -> Result<String> {
        let context_path = "global_context.json";
        match fs::read_to_string(context_path) {
            Ok(content) => {
                let system_prompt = format!(
                    "You are an AI assistant specialized in analyzing financial and legal documents. \
                    You have access to a comprehensive document collection for a client. \
                    Here is the complete context of the client's file:\n\n{}\n\n\
                    IMPORTANT INSTRUCTIONS:\n\
                    - When referencing any document, ALWAYS include the document ID and document version ID (documentId and documentVersionId fields) in your response\n\
                    - Format document references like: [Document ID: 68371449b15db0ce743c25b3, Document Version ID: 68371449b15db0ce743c25b3]\n\
                    - If discussing multiple documents, list all relevant document IDs\n\
                    - Use this information to provide accurate, detailed responses about the client's",
                    content
                );
                Ok(system_prompt)
            }
            Err(e) => {
                println!("⚠️  Warning: Could not load global_context.json: {}", e);
                Ok("You are an AI assistant specialized in analyzing financial and legal documents. Always include document IDs when referencing specific documents.".to_string())
            }
        }
    }
    
    pub async fn send_message_with_system(&self, messages: &[Message], system: Option<String>) -> Result<String> {
        let request = ClaudeRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: 4096,
            messages: messages.to_vec(),
            temperature: 0.7,
            system,
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