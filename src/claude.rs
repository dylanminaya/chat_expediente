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
    #[serde(rename = "document")]
    Document {
        #[serde(rename = "source")]
        source: DocumentSource,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocumentSource {
    #[serde(rename = "type")]
    pub source_type: String, // "base64"
    #[serde(rename = "media_type")]
    pub media_type: String, // "application/pdf"
    pub data: String, // base64 encoded content
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
                    - Use this information to provide accurate, detailed responses about the client's\n\
                    - Please respond in the same language as the question or inquiry.",
                    content
                );
                Ok(system_prompt)
            }
            Err(e) => {
                println!("⚠️  Warning: Could not load global_context.json: {}", e);
                Ok("You are an AI assistant specialized in analyzing financial and legal documents. \
                Always include document IDs when referencing specific documents. \
                IMPORTANT: When referencing documents, always include both Document ID and Document Version ID in this exact format: \
                Document ID: [24-character hex ID] \
                Document Version ID: [24-character hex ID] \
                Use this format regardless of the language you respond in.".to_string())
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
        println!("🔍 Request body size: {} bytes", body.len());
        
        // Retry logic for rate limiting
        let mut retry_count = 0;
        let max_retries = 3;
        
        loop {
            let blob = Blob::new(body.as_bytes());
            
            let result = self.client
                .invoke_model()
                .model_id(&self.model_id)
                .content_type("application/json")
                .body(blob)
                .send()
                .await;
            
            match result {
                Ok(response) => {
                    let response_body = response.body().as_ref();
                    let response_text = String::from_utf8_lossy(response_body);
                    
                    let claude_response: ClaudeResponse = serde_json::from_slice(response_body)
                        .map_err(|e| anyhow!("Failed to parse response: {} - Response body: {}", e, response_text))?;
                    
                    if let Some(content) = claude_response.content.first() {
                        let input_tokens = claude_response.usage.input_tokens;
                        let output_tokens = claude_response.usage.output_tokens;
                        let total_tokens = input_tokens + output_tokens;
                        
                        println!("📊 TOKEN USAGE SUMMARY:");
                        println!("   Input tokens:  {}", input_tokens);
                        println!("   Output tokens: {}", output_tokens);
                        println!("   Total tokens:  {}", total_tokens);
                        println!("   💰 Estimated cost: ~${:.4} USD", (total_tokens as f64) * 0.00025 / 1000.0);
                        
                        if input_tokens > 30000 {
                            println!("⚠️  WARNING: High input token usage! Consider reducing content size.");
                        }
                        if total_tokens > 40000 {
                            println!("🚨 ALERT: Very high token usage! You may hit rate limits.");
                        }
                        
                        return Ok(content.text.clone());
                    } else {
                        return Err(anyhow!("No content in response"));
                    }
                }
                Err(e) => {
                    println!("❌ AWS Bedrock error details: {:?}", e);
                    
                    // Check if it's a throttling error
                    let error_string = format!("{:?}", e);
                    if error_string.contains("ThrottlingException") || error_string.contains("Too many tokens") {
                        if retry_count < max_retries {
                            retry_count += 1;
                            let wait_time = 2_u64.pow(retry_count) * 1000; // Exponential backoff: 2s, 4s, 8s
                            println!("🔄 Rate limited! Retrying in {}ms (attempt {}/{})", wait_time, retry_count, max_retries);
                            tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
                            continue;
                        } else {
                            return Err(anyhow!("Rate limited after {} retries. Please wait before trying again.", max_retries));
                        }
                    } else {
                        return Err(anyhow!("Failed to invoke model: {}", e));
                    }
                }
            }
        }
    }
} 