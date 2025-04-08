//! # MCPRS: Model Context Protocol para Rust
//!
//! MCPRS é uma biblioteca que implementa o Model Context Protocol (MCP), um protocolo
//! padronizado para comunicação com diferentes serviços de modelos de linguagem (LLMs) e IA.
//!
//! ## Funcionalidades Principais
//!
//! - **Protocolo Padronizado**: Define um formato comum para mensagens trocadas com diferentes LLMs
//! - **Sistema de Agentes**: Abstração para integrações com diferentes APIs (OpenAI, DeepSeek, etc.)
//! - **Servidor e Cliente**: Implementações prontas para uso em aplicações
//! - **Autenticação**: Sistema de autenticação baseado em tokens
//! - **Histórico de Conversas**: Gerenciamento de contexto e histórico
//! - **Streaming**: Suporte para respostas incrementais em streaming
//!
//! ## Guia Rápido
//!
//! ### Cliente Simples
//!
//! ```rust,no_run
//! use mcprs::client::{create_mcp_message_for_agent, send_mcp_request};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar uma mensagem para o OpenAI
//! let message = create_mcp_message_for_agent(
//!     "openai",
//!     "chat",
//!     json!({
//!         "user_prompt": "Explique o que é Rust em poucas palavras"
//!     })
//! );
//!
//! // Enviar para um servidor MCP
//! let response = send_mcp_request("http://localhost:3000/mcp", &message).await?;
//!
//! println!("Resposta: {}", response.payload["answer"]);
//! # Ok(())
//! # }
//! ```
//!
//! ### Servidor Básico
//!
//! ```rust,no_run
//! use mcprs::agent::AgentRegistry;
//! use mcprs::agent_openai::create_openai_agent;
//! use mcprs::server::run_http_server;
//! use std::net::SocketAddr;
//!
//! # async fn example() {
//! // Configurar variável de ambiente
//! std::env::set_var("OPENAI_API_KEY", "sua-chave-aqui");
//!
//! // Criar e configurar registro
//! let mut registry = AgentRegistry::new();
//! registry.register_agent(Box::new(create_openai_agent(None)));
//!
//! // Iniciar servidor
//! let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//! run_http_server(registry, addr).await;
//! # }
//! ```
//!
//! ## Módulos Principais
//!
//! - [`agent`]: Define a trait AIAgent e estruturas básicas do protocolo
//! - [`server`]: Implementação do servidor HTTP para processar requisições MCP
//! - [`client`]: Funções para enviar requisições MCP
//! - [`agent_openai`]: Implementação de agente para a API OpenAI
//! - [`agent_deepseek`]: Implementação de agente para a API DeepSeek
//! - [`auth`]: Sistema de autenticação para o servidor
//! - [`conversation`]: Gerenciamento de histórico de conversas
//! - [`streaming`]: Suporte para respostas em streaming

pub mod agent;
pub mod agent_deepseek;
pub mod agent_openai;
pub mod auth;
pub mod client;
pub mod conversation;
pub mod server;
pub mod streaming;
pub mod testing;

/// Re-exporta tipos comumente usados para facilitar o uso
pub use agent::{AIAgent, AgentRegistry, MCPError, MCPMessage};
pub use auth::{AuthConfig, AuthUser};
pub use conversation::{Conversation, ConversationManager, ConversationMessage};
pub use streaming::{StreamingToken, TokenStream};

// Exportação adicional de funções do servidor
pub use server::run_http_server;
pub use server::run_http_server_with_auth;
