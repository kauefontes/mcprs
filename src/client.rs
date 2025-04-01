use crate::agent::MCPMessage;
use reqwest::Client;
use thiserror::Error;

/// Erros possíveis no client.
#[derive(Error, Debug)]
pub enum MCPClientError {
    #[error("Erro de rede ao enviar requisição: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("O servidor retornou um status inesperado: {0}")]
    UnexpectedStatus(reqwest::StatusCode),

    #[error("Falha ao deserializar a resposta MCP: {0}")]
    DeserializationError(String),
}

/// Envia uma requisição MCP via HTTP POST para um determinado endpoint /mcp.
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
/// O campo `command` segue o formato "agente:acao".
pub fn create_mcp_message_for_agent(
    agent: &str,
    action: &str,
    payload: serde_json::Value,
) -> MCPMessage {
    let command = format!("{}:{}", agent, action);
    MCPMessage::new(&command, payload)
}
