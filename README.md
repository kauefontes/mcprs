# MCPRS: Model Context Protocol para Rust

[![Crates.io](https://img.shields.io/crates/v/mcprs)](https://crates.io/crates/mcprs)
[![Documentation](https://docs.rs/mcprs/badge.svg)](https://docs.rs/mcprs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCPRS é uma biblioteca Rust que implementa um protocolo padronizado (Model Context Protocol) para comunicação com diversos Large Language Models (LLMs) e serviços de IA. Ela fornece uma camada de abstração unificada que permite aos desenvolvedores interagir com diferentes APIs de IA (como OpenAI GPT, DeepSeek, etc.) de forma consistente e intercambiável.

## Principais Características

- 🔄 **Interface Unificada**: Uma única API consistente para todos os modelos de IA
- 🌐 **Múltiplos Provedores**: Suporte integrado para OpenAI e DeepSeek (facilmente extensível)
- 🔌 **Arquitetura Plugável**: Adicione novos provedores de IA implementando a trait `AIAgent`
- 🔒 **Autenticação**: Sistema de autenticação baseado em tokens
- 💬 **Gerenciamento de Conversas**: Armazene e gerencie histórico de conversas
- 📊 **Streaming**: Suporte para respostas em streaming dos modelos
- 🧪 **Testabilidade**: Abstrações para facilitar testes com mocks

## Instalação

Adicione MCPRS ao seu `Cargo.toml`:

```toml
[dependencies]
mcprs = "0.1.0"
```

## Guia Rápido

### Cliente Simples

```rust
use mcprs::client::{create_mcp_message_for_agent, send_mcp_request};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar uma mensagem para o agente OpenAI
    let message = create_mcp_message_for_agent(
        "openai",
        "chat",
        json!({
            "user_prompt": "Explique a linguagem Rust em poucas palavras"
        }),
    );

    // Enviar para um servidor MCP
    let response = send_mcp_request("http://localhost:3000/mcp", &message).await?;

    // Processar a resposta
    println!("Resposta: {}", response.payload["answer"]);

    Ok(())
}
```

### Servidor Básico

```rust
use mcprs::agent::AgentRegistry;
use mcprs::agent_openai::create_openai_agent;
use mcprs::server::run_http_server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Configurar variável de ambiente com sua chave de API
    std::env::set_var("OPENAI_API_KEY", "sua-chave-aqui");

    // Criar e configurar o registro de agentes
    let mut registry = AgentRegistry::new();
    registry.register_agent(Box::new(create_openai_agent(None)));

    // Iniciar o servidor
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    run_http_server(registry, addr).await;
}
```

### Servidor Avançado com Autenticação e Histórico

```rust
use mcprs::agent::AgentRegistry;
use mcprs::agent_openai::create_openai_agent;
use mcprs::auth::AuthConfig;
use mcprs::conversation::ConversationManager;
use mcprs::server::run_http_server_with_auth;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Configurar variável de ambiente
    std::env::set_var("OPENAI_API_KEY", "sua-chave-aqui");

    // Configurar agentes
    let mut registry = AgentRegistry::new();
    registry.register_agent(Box::new(create_openai_agent(None)));

    // Configurar autenticação
    let auth_config = AuthConfig::new();
    auth_config.add_token("token-de-acesso-seguro".to_string());

    // Configurar gerenciamento de conversas (24h de retenção)
    let conversation_manager = ConversationManager::new(24);

    // Iniciar servidor
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    run_http_server_with_auth(registry, auth_config, conversation_manager, addr).await;
}
```

## Conceitos Principais

### Protocolo MCP

O Model Context Protocol (MCP) define um formato padronizado para mensagens trocadas entre clientes e servidores:

```rust
pub struct MCPMessage {
    pub magic: String,      // Identificador do protocolo "MCP0"
    pub version: u8,        // Versão do protocolo
    pub command: String,    // Comando no formato "agente:ação"
    pub payload: Value,     // Dados JSON da requisição/resposta
}
```

### Agentes de IA

Os agentes implementam a trait `AIAgent` e encapsulam a comunicação com serviços específicos de IA:

```rust
#[async_trait]
pub trait AIAgent: Send + Sync {
    fn name(&self) -> &str;
    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError>;
}
```

Agentes disponíveis:
- **DummyAgent**: Para testes, apenas ecoa o payload recebido
- **OpenAIAgent**: Integra com a API do OpenAI (ChatGPT)
- **DeepSeekAgent**: Integra com a API DeepSeek

## Documentação Detalhada

### Cliente

O módulo `client` fornece funções para construir e enviar mensagens MCP:

```rust
// Criar uma mensagem MCP formatada corretamente
let message = create_mcp_message_for_agent("openai", "chat", payload);

// Enviar a mensagem para um servidor MCP
let response = send_mcp_request("http://exemplo.com/mcp", &message).await?;
```

### Servidor

O módulo `server` implementa um servidor HTTP que pode receber e processar mensagens MCP:

```rust
// Versão básica do servidor
run_http_server(registry, addr).await;

// Versão avançada com autenticação e histórico
run_http_server_with_auth(registry, auth_config, conversation_manager, addr).await;
```

### Autenticação

O módulo `auth` fornece um sistema de autenticação baseado em tokens:

```rust
// Configuração de tokens permitidos
let auth_config = AuthConfig::new();
auth_config.add_token("token-secreto".to_string());

// No cliente (usando reqwest)
client.post(url).bearer_auth("token-secreto").json(&message).send().await?;
```

### Conversações

O módulo `conversation` implementa gerenciamento de histórico de conversações:

```rust
// Criar um gerenciador de conversas
let manager = ConversationManager::new(24); // retenção de 24 horas

// Criar uma nova conversa
let conversation = manager.create_conversation()?;

// Adicionar mensagens
manager.add_message_to_conversation(&conversation.id, "user", "Olá!")?;

// Recuperar histórico
let history = manager.get_conversation(&conversation.id);
```

### Streaming

O módulo `streaming` fornece suporte para processamento de respostas em streaming:

```rust
// Processar um stream de bytes em tokens estruturados
let token_stream = process_json_stream::<_, ResponseType>(bytes_stream).await?;

// Consumir os tokens
while let Some(Ok(token)) = token_stream.next().await {
    if token.is_finish {
        break;
    }
    println!("{}", token.content);
}
```

## Implementando um Novo Agente

Para adicionar suporte a um novo serviço de IA, implemente a trait `AIAgent`:

```rust
pub struct MyNewAgent {
    pub api_key: String,
    http_client: Box<dyn HttpClient>,
}

#[async_trait]
impl AIAgent for MyNewAgent {
    fn name(&self) -> &str {
        "my-agent"
    }

    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        // Implemente a lógica de comunicação com a API externa
        // ...

        Ok(MCPMessage::new(
            "my-agent_response",
            json!({ "answer": "Resposta processada" }),
        ))
    }
}
```

## Configuração de Ambiente

O projeto utiliza variáveis de ambiente para configurações sensíveis:

- `OPENAI_API_KEY` - Chave de API para o agente OpenAI
- `DEEPSEEK_API_KEY` - Chave de API para o agente DeepSeek
- `DEEPSEEK_ENDPOINT` - URL do endpoint DeepSeek (padrão: https://api.deepseek.ai)
- `DEEPSEEK_MODEL` - Modelo DeepSeek a ser usado (padrão: deepseek-chat)

## Exemplos

O repositório inclui exemplos completos:

- **Client**: Um cliente simples para enviar requisições
- **Server**: Um servidor completo com autenticação e histórico
- **Demonstrate Agents**: Exemplo que cria um servidor e envia requisições para testar

## Testes

A biblioteca vem com uma suíte de testes automatizados:

```bash
# Executar todos os testes
cargo test

# Executar testes específicos
cargo test agent_
```

## Próximos Passos e Contribuições

Contribuições são bem-vindas! Áreas de melhoria incluem:

- Adicionar mais agentes de IA (Claude, Cohere, Mistral, etc.)
- Melhorar o sistema de streaming com tipagem específica por agente
- Implementar cache de respostas
- Adicionar ferramentas (tools) para processamento avançado

## Licença

Este projeto está licenciado sob a [Licença MIT](LICENSE).
