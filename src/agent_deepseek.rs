use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

use crate::agent::{AIAgent, MCPError, MCPMessage};

/// Exemplo de agente que se integra à API DeepSeek (hipotético).
/// Em produção, adapte para suas funções/métodos, endpoints, etc.
pub struct DeepSeekAgent {
    pub api_key: String,
    pub endpoint: String,
}

#[derive(Serialize)]
struct DeepSeekRequest {
    query: String,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    result: String,
}

#[async_trait]
impl AIAgent for DeepSeekAgent {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        // Supondo que no payload tenha { "question": "Quem descobriu o Brasil?" }
        let question = message
            .payload
            .get("question")
            .and_then(Value::as_str)
            .unwrap_or("Pergunta vazia");

        let request_body = DeepSeekRequest {
            query: question.to_string(),
        };

        let client = Client::new();
        let resp = client
            .post(&format!("{}/api/v1/ask", &self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(MCPError::InternalAgentError(format!(
                "DeepSeek retornou status {}",
                resp.status()
            )));
        }

        let parsed_resp = resp
            .json::<DeepSeekResponse>()
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        // Monta MCPMessage de resposta
        Ok(MCPMessage::new(
            "deepseek_response",
            json!({ "answer": parsed_resp.result }),
        ))
    }
}

/// Função auxiliar para facilitar a criação do agente.
pub fn create_deepseek_agent() -> DeepSeekAgent {
    DeepSeekAgent {
        api_key: env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "SUA_DEEPSEEK_KEY".to_string()),
        endpoint: env::var("DEEPSEEK_ENDPOINT")
            .unwrap_or_else(|_| "https://api.deepseek.ai".to_string()),
    }
}
