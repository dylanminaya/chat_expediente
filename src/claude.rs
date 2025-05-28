use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{Client, primitives::Blob};
use aws_types::region::Region;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ClaudeRequest {
    anthropic_version: String,
    max_tokens: u32,
    messages: Vec<Message>,
    temperature: f32,
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
    
    pub async fn send_message(&self, messages: &[Message]) -> Result<String> {
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