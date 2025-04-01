use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Possíveis erros referentes ao processamento de mensagens MCP.
#[derive(Error, Debug)]
pub enum MCPError {
    #[error("Formato de comando inválido (esperado 'agente:acao')")]
    InvalidCommandFormat,

    #[error("Agente '{0}' não foi encontrado no registro")]
    AgentNotRegistered(String),

    #[error("Erro interno do agente: {0}")]
    InternalAgentError(String),
}

/// Estrutura de dados que representa uma mensagem MCP.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MCPMessage {
    pub magic: String,
    pub version: u8,
    pub command: String,
    pub payload: Value,
}

impl MCPMessage {
    /// Cria uma nova mensagem MCP com os valores padrão para `magic` e `version`.
    pub fn new(command: &str, payload: Value) -> Self {
        MCPMessage {
            magic: "MCP0".to_string(),
            version: 1,
            command: command.to_string(),
            payload,
        }
    }
}

/// Trait que define o comportamento básico esperado de um agente de IA.
///
/// Qualquer agente deve ser capaz de:
/// 1. Informar seu nome (chave de identificação).
/// 2. Receber uma `MCPMessage` e retornar outra `MCPMessage` ou erro de processamento.
#[async_trait]
pub trait AIAgent: Send + Sync {
    /// Retorna o nome identificador do agente.
    fn name(&self) -> &str;

    /// Processa uma requisição MCP e retorna uma resposta.
    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError>;
}

/// Estrutura para gerenciar múltiplos agentes de IA.
pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn AIAgent>>,
}

impl AgentRegistry {
    /// Cria um novo registro vazio de agentes.
    pub fn new() -> Self {
        AgentRegistry {
            agents: HashMap::new(),
        }
    }

    /// Registra um novo agente no registro.
    pub fn register_agent(&mut self, agent: Box<dyn AIAgent>) {
        self.agents.insert(agent.name().to_string(), agent);
    }

    /// Processa uma mensagem roteando-a para o agente correto.
    /// O comando deve estar no formato "nomeAgente:acao".
    pub async fn process(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        let parts: Vec<&str> = message.command.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(MCPError::InvalidCommandFormat);
        }
        let agent_key = parts[0];
        if let Some(agent) = self.agents.get(agent_key) {
            agent.process_request(message).await
        } else {
            Err(MCPError::AgentNotRegistered(agent_key.to_string()))
        }
    }
}

/// Um agente simples (DummyAgent) que apenas replica o payload recebido.
/// Exemplo de como implementar a trait `AIAgent`.
pub struct DummyAgent {
    pub api_key: String,
}

#[async_trait]
impl AIAgent for DummyAgent {
    fn name(&self) -> &str {
        "dummy"
    }

    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        // Neste exemplo, apenas ecoamos o payload recebido, mas poderíamos chamar APIs externas.
        Ok(MCPMessage::new("dummy_response", message.payload))
    }
}
