//! # Módulo de Agentes
//!
//! Este módulo define a interface central para agentes de inteligência artificial
//! no protocolo MCP, incluindo as estruturas básicas de mensagens, erros, e
//! o sistema de registro de agentes.
//!
//! ## Componentes Principais
//!
//! - [`AIAgent`]: Trait que define o comportamento de um agente de IA
//! - [`AgentRegistry`]: Gerenciador central para múltiplos agentes
//! - [`MCPMessage`]: Formato padrão de mensagem para o protocolo MCP
//! - [`MCPError`]: Tipos de erros específicos do protocolo
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::agent::{AgentRegistry, DummyAgent, MCPMessage};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar o registro de agentes
//! let mut registry = AgentRegistry::new();
//!
//! // Registrar um agente
//! registry.register_agent(Box::new(DummyAgent {
//!     api_key: "dummy_key".to_string(),
//! }));
//!
//! // Criar uma mensagem para o agente
//! let message = MCPMessage::new("dummy:echo", json!({"text": "Hello, world!"}));
//!
//! // Processar a mensagem
//! let response = registry.process(message).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Erros que podem ocorrer durante o processamento de mensagens MCP.
///
/// Usado para representar falhas específicas ao protocolo MCP que podem
/// acontecer durante a comunicação com agentes ou o processamento de mensagens.
#[derive(Error, Debug)]
pub enum MCPError {
    /// Retornado quando o formato de comando não segue o padrão "agente:acao".
    #[error("Formato de comando inválido (esperado 'agente:acao')")]
    InvalidCommandFormat,

    /// Retornado quando o registro não tem o agente solicitado.
    #[error("Agente '{0}' não foi encontrado no registro")]
    AgentNotRegistered(String),

    /// Retornado quando ocorre um erro interno em um agente específico.
    #[error("Erro interno do agente: {0}")]
    InternalAgentError(String),
}

/// Estrutura central que representa uma mensagem no protocolo MCP.
///
/// Cada mensagem contém:
/// - Um campo `magic` para identificar o protocolo ("MCP0")
/// - Um número de `version` para controle de compatibilidade
/// - Um `command` que segue o formato "agente:acao"
/// - Um `payload` que contém os dados JSON da requisição ou resposta
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MCPMessage {
    /// Identificador do protocolo, deve ser "MCP0"
    pub magic: String,

    /// Versão do protocolo, atualmente 1
    pub version: u8,

    /// Comando no formato "agente:acao"
    pub command: String,

    /// Payload JSON com dados da requisição ou resposta
    pub payload: Value,
}

impl MCPMessage {
    /// Cria uma nova mensagem MCP com os valores padrão para `magic` e `version`.
    ///
    /// # Argumentos
    /// * `command` - Comando no formato "agente:acao"
    /// * `payload` - Dados JSON da requisição ou resposta
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::agent::MCPMessage;
    /// use serde_json::json;
    ///
    /// let message = MCPMessage::new("openai:chat", json!({"user_prompt": "Hello"}));
    /// assert_eq!(message.magic, "MCP0");
    /// assert_eq!(message.version, 1);
    /// ```
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
/// 2. Receber uma `MCPMessage` e processar, retornando outra `MCPMessage` ou um erro.
///
/// Para implementar um novo agente, é necessário implementar esta trait.
#[async_trait]
pub trait AIAgent: Send + Sync {
    /// Retorna o nome identificador do agente.
    ///
    /// Este nome é usado como prefixo no campo `command` das mensagens MCP.
    fn name(&self) -> &str;

    /// Processa uma requisição MCP e retorna uma resposta.
    ///
    /// # Argumentos
    /// * `message` - A mensagem MCP recebida para processamento
    ///
    /// # Retorna
    /// * `Ok(MCPMessage)` - A resposta processada com sucesso
    /// * `Err(MCPError)` - Um erro que ocorreu durante o processamento
    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError>;
}

/// Estrutura para gerenciar múltiplos agentes de IA.
///
/// O `AgentRegistry` mantém uma coleção de agentes e roteia mensagens para o
/// agente apropriado com base no prefixo do comando.
pub struct AgentRegistry {
    /// Mapa de nome do agente para sua implementação
    agents: HashMap<String, Box<dyn AIAgent>>,
}

impl AgentRegistry {
    /// Cria um novo registro vazio de agentes.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::agent::AgentRegistry;
    ///
    /// let registry = AgentRegistry::new();
    /// ```
    pub fn new() -> Self {
        AgentRegistry {
            agents: HashMap::new(),
        }
    }

    /// Registra um novo agente no registro.
    ///
    /// # Argumentos
    /// * `agent` - O agente a ser registrado, como um objeto Box<dyn AIAgent>
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::agent::{AgentRegistry, DummyAgent};
    ///
    /// let mut registry = AgentRegistry::new();
    /// registry.register_agent(Box::new(DummyAgent {
    ///     api_key: "dummy_key".to_string(),
    /// }));
    /// ```
    pub fn register_agent(&mut self, agent: Box<dyn AIAgent>) {
        self.agents.insert(agent.name().to_string(), agent);
    }

    /// Processa uma mensagem roteando-a para o agente correto.
    ///
    /// O comando deve estar no formato "nomeAgente:acao". A parte "nomeAgente"
    /// é usada para localizar o agente apropriado no registro.
    ///
    /// # Argumentos
    /// * `message` - A mensagem a ser processada
    ///
    /// # Retorna
    /// * `Ok(MCPMessage)` - A resposta do agente
    /// * `Err(MCPError)` - Erro que ocorreu durante o processamento ou roteamento
    ///
    /// # Erros
    /// * `MCPError::InvalidCommandFormat` - Se o comando não seguir o formato "agente:acao"
    /// * `MCPError::AgentNotRegistered` - Se o agente especificado não estiver registrado
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
///
/// Este agente é útil para testes e demonstrações, já que não requer conexão
/// com serviços externos.
pub struct DummyAgent {
    /// Chave de API simulada (não utilizada realmente)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mcpmessage_new() {
        let msg = MCPMessage::new("test:command", json!({"key": "value"}));
        assert_eq!(msg.magic, "MCP0");
        assert_eq!(msg.version, 1);
        assert_eq!(msg.command, "test:command");
        assert_eq!(msg.payload, json!({"key": "value"}));
    }

    #[tokio::test]
    async fn test_dummy_agent() {
        let agent = DummyAgent {
            api_key: "test_key".to_string(),
        };

        let msg = MCPMessage::new("dummy:test", json!({"echo": "this"}));
        let result = agent.process_request(msg).await.unwrap();

        assert_eq!(result.command, "dummy_response");
        assert_eq!(result.payload, json!({"echo": "this"}));
    }

    #[tokio::test]
    async fn test_registry_routing() {
        let mut registry = AgentRegistry::new();
        registry.register_agent(Box::new(DummyAgent {
            api_key: "test_key".to_string(),
        }));

        // Teste de roteamento bem-sucedido
        let msg = MCPMessage::new("dummy:action", json!({"test": true}));
        let result = registry.process(msg).await.unwrap();
        assert_eq!(result.command, "dummy_response");

        // Teste de agente não encontrado
        let msg2 = MCPMessage::new("unknown:action", json!({}));
        let err = registry.process(msg2).await.unwrap_err();
        assert!(matches!(err, MCPError::AgentNotRegistered(s) if s == "unknown"));

        // Teste de formato inválido
        let msg3 = MCPMessage::new("invalid-format", json!({}));
        let err = registry.process(msg3).await.unwrap_err();
        assert!(matches!(err, MCPError::InvalidCommandFormat));
    }
}
