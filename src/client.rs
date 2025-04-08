//! # Módulo de Cliente MCP
//!
//! Este módulo fornece funções para criar e enviar mensagens MCP para
//! servidores compatíveis. Ele simplifica a comunicação com servidores MCP
//! abstraindo os detalhes do protocolo HTTP subjacente.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::client::{create_mcp_message_for_agent, send_mcp_request};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar uma mensagem MCP para o agente OpenAI
//! let message = create_mcp_message_for_agent(
//!     "openai",
//!     "chat",
//!     json!({
//!         "user_prompt": "Explique o que é Rust em uma frase"
//!     })
//! );
//!
//! // Enviar a requisição para um servidor MCP
//! let response = send_mcp_request("http://localhost:3000/mcp", &message).await?;
//!
//! // Processar a resposta
//! println!("Resposta: {}", response.payload["answer"]);
//! # Ok(())
//! # }
//! ```

use crate::agent::MCPMessage;
use reqwest::Client;
use thiserror::Error;

/// Erros que podem ocorrer durante o envio de requisições MCP.
#[derive(Error, Debug)]
pub enum MCPClientError {
    /// Erro de rede ao enviar a requisição
    #[error("Erro de rede ao enviar requisição: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// O servidor retornou um status inesperado
    #[error("O servidor retornou um status inesperado: {0}")]
    UnexpectedStatus(reqwest::StatusCode),

    /// Falha ao deserializar a resposta MCP
    #[error("Falha ao deserializar a resposta MCP: {0}")]
    DeserializationError(String),
}

/// Envia uma requisição MCP via HTTP POST para um servidor.
///
/// # Argumentos
/// * `server_url` - URL do endpoint MCP (geralmente termina com `/mcp`)
/// * `message` - A mensagem MCP a ser enviada
///
/// # Retorna
/// * `Ok(MCPMessage)` - A resposta MCP do servidor
/// * `Err(MCPClientError)` - Se ocorrer algum erro na comunicação
///
/// # Exemplo
///
/// ```rust,no_run
/// use mcprs::agent::MCPMessage;
/// use mcprs::client::send_mcp_request;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let message = MCPMessage::new("openai:chat", json!({"user_prompt": "Olá!"}));
/// let response = send_mcp_request("http://localhost:3000/mcp", &message).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_mcp_request(
    server_url: &str,
    message: &MCPMessage,
) -> Result<MCPMessage, MCPClientError> {
    let client = Client::new();
    let response = client.post(server_url).json(&message).send().await?;

    if !response.status().is_success() {
        return Err(MCPClientError::UnexpectedStatus(response.status()));
    }

    let mcp_resp = response
        .json::<MCPMessage>()
        .await
        .map_err(|e| MCPClientError::DeserializationError(e.to_string()))?;

    Ok(mcp_resp)
}

/// Cria uma mensagem MCP específica para um agente e ação.
///
/// Esta função facilita a criação de mensagens MCP no formato correto,
/// definindo o campo `command` como "{agente}:{acao}".
///
/// # Argumentos
/// * `agent` - Nome do agente (ex: "openai", "deepseek")
/// * `action` - Ação a ser executada (ex: "chat", "complete")
/// * `payload` - Payload JSON com os dados da requisição
///
/// # Retorna
/// Uma nova instância de `MCPMessage` configurada
///
/// # Exemplo
///
/// ```rust
/// use mcprs::client::create_mcp_message_for_agent;
/// use serde_json::json;
///
/// let message = create_mcp_message_for_agent(
///     "openai",
///     "chat",
///     json!({
///         "user_prompt": "Explique Rust brevemente",
///         "temperature": 0.7
///     })
/// );
///
/// assert_eq!(message.command, "openai:chat");
/// assert_eq!(message.magic, "MCP0");
/// ```
pub fn create_mcp_message_for_agent(
    agent: &str,
    action: &str,
    payload: serde_json::Value,
) -> MCPMessage {
    let command = format!("{}:{}", agent, action);
    MCPMessage::new(&command, payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_mcp_message_for_agent() {
        let message = create_mcp_message_for_agent(
            "openai",
            "chat",
            json!({
                "user_prompt": "Teste",
                "temperature": 0.5
            }),
        );

        assert_eq!(message.command, "openai:chat");
        assert_eq!(message.magic, "MCP0");
        assert_eq!(message.version, 1);
        assert_eq!(message.payload["user_prompt"], "Teste");
        assert_eq!(message.payload["temperature"], 0.5);
    }

    // Testes para send_mcp_request seriam mais complexos e
    // necessitariam de um servidor de mock, o que está fora
    // do escopo destes testes unitários simples.
}
