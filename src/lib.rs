/// # MCP Library
///
/// Biblioteca geral para comunicação via Model Context Protocol (MCP).
/// Define estruturas, traits e métodos básicos para criação de agentes de IA,
/// roteamento e chamadas HTTP de envio/recebimento.
///
/// ## Módulos
///
/// - `agent`: Define o comportamento de agentes de IA e um registro centralizado.
/// - `agent_openai`: Agente de exemplo que integra com a API do OpenAI (ChatGPT).
/// - `agent_deepseek`: Agente de exemplo que integra com a API do DeepSeek.
/// - `server`: Implementa servidor HTTP para receber e processar mensagens MCP.
/// - `client`: Fornece funções para enviar requisições MCP a um servidor.
pub mod agent;
pub mod agent_deepseek;
pub mod agent_openai;
pub mod client;
pub mod server;
pub mod testing;
