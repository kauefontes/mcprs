use crate::agent::{AIAgent, MCPError, MCPMessage};
use crate::testing::HttpClient;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;

pub struct OpenAIAgent {
    pub api_key: String,
    pub model: String,
    http_client: Box<dyn HttpClient>,
}

impl OpenAIAgent {
    pub fn new(api_key: String, model: String, http_client: Box<dyn HttpClient>) -> Self {
        Self {
            api_key,
            model,
            http_client,
        }
    }
}

#[derive(serde::Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
}

#[derive(serde::Serialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatChoice>,
}

#[derive(serde::Deserialize)]
struct OpenAIChatChoice {
    message: OpenAIChatMessageResponse,
}

#[derive(serde::Deserialize)]
struct OpenAIChatMessageResponse {
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[async_trait]
impl AIAgent for OpenAIAgent {
    fn name(&self) -> &str {
        "openai"
    }

    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        let user_prompt = message
            .payload
            .get("user_prompt")
            .and_then(Value::as_str)
            .ok_or_else(|| MCPError::InternalAgentError("Missing user_prompt".to_string()))?;

        let request_body = OpenAIChatRequest {
            model: self.model.clone(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
        };

        let headers = vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", self.api_key),
            ),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let response = self
            .http_client
            .post(
                "https://api.openai.com/v1/chat/completions".to_string(),
                serde_json::to_vec(&request_body)
                    .map_err(|e| MCPError::InternalAgentError(e.to_string()))?,
                headers,
            )
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MCPError::InternalAgentError(format!(
                "OpenAI API retornou status {}",
                response.status()
            )));
        }

        let resp_json = response
            .json::<OpenAIChatResponse>()
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        let answer_text = resp_json
            .choices
            .get(0)
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| MCPError::InternalAgentError("No response choices".to_string()))?;

        Ok(MCPMessage::new(
            "openai_response",
            json!({ "answer": answer_text }),
        ))
    }
}

/// Factory function atualizada para criar o agente com HTTP client
pub fn create_openai_agent(http_client: Option<Box<dyn HttpClient>>) -> OpenAIAgent {
    let client = http_client.unwrap_or_else(|| Box::new(crate::testing::ReqwestClient::new()));

    OpenAIAgent::new(
        env::var("OPENAI_API_KEY").unwrap_or_else(|_| "SUA_KEY_AQUI".to_string()),
        "gpt-3.5-turbo".to_string(),
        client,
    )
}
