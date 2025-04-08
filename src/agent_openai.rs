//! # Agente para integração com a API OpenAI
//!
//! Este módulo implementa um agente que se comunica com a API do OpenAI,
//! permitindo enviar prompts para modelos como GPT-3.5-Turbo, GPT-4, etc.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::agent::{AgentRegistry, MCPMessage};
//! use mcprs::agent_openai::create_openai_agent;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configurar variável de ambiente (ou diretamente na criação do agente)
//! std::env::set_var("OPENAI_API_KEY", "sua-chave-aqui");
//!
//! // Criar e registrar o agente OpenAI
//! let mut registry = AgentRegistry::new();
//! registry.register_agent(Box::new(create_openai_agent(None)));
//!
//! // Criar uma mensagem para o modelo do OpenAI
//! let message = MCPMessage::new(
//!     "openai:chat",
//!     json!({
//!         "user_prompt": "Explique o que é Rust em poucas palavras"
//!     })
//! );
//!
//! // Processar a mensagem
//! let response = registry.process(message).await?;
//! println!("Resposta: {}", response.payload["answer"]);
//! # Ok(())
//! # }
//! ```

use crate::agent::{AIAgent, MCPError, MCPMessage};
use crate::testing::HttpClient;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;

/// Agente para comunicação com a API OpenAI.
///
/// Este agente implementa a trait `AIAgent` e se conecta aos endpoints da
/// OpenAI para enviar prompts e obter respostas de modelos como GPT-3.5 e GPT-4.
pub struct OpenAIAgent {
    /// Chave de API do OpenAI
    pub api_key: String,

    /// Nome do modelo a ser usado (ex: "gpt-3.5-turbo", "gpt-4")
    pub model: String,

    /// Cliente HTTP para fazer as requisições
    http_client: Box<dyn HttpClient>,
}

impl OpenAIAgent {
    /// Cria uma nova instância do agente OpenAI.
    ///
    /// # Argumentos
    /// * `api_key` - Chave de API da OpenAI
    /// * `model` - Nome do modelo a ser usado
    /// * `http_client` - Cliente HTTP para fazer as requisições
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::agent_openai::OpenAIAgent;
    /// use mcprs::testing::ReqwestClient;
    ///
    /// let agent = OpenAIAgent::new(
    ///     "sua-chave-api".to_string(),
    ///     "gpt-3.5-turbo".to_string(),
    ///     Box::new(ReqwestClient::new())
    /// );
    /// ```
    pub fn new(api_key: String, model: String, http_client: Box<dyn HttpClient>) -> Self {
        Self {
            api_key,
            model,
            http_client,
        }
    }
}

/// Estrutura para o corpo da requisição à API OpenAI Chat
#[derive(serde::Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
}

/// Estrutura para uma mensagem na requisição à API OpenAI Chat
#[derive(serde::Serialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

/// Estrutura para a resposta da API OpenAI Chat
#[derive(serde::Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatChoice>,
}

/// Estrutura para um item de escolha na resposta da API OpenAI
#[derive(serde::Deserialize)]
struct OpenAIChatChoice {
    message: OpenAIChatMessageResponse,
}

/// Estrutura para a mensagem dentro de um item de escolha na resposta
#[derive(serde::Deserialize)]
struct OpenAIChatMessageResponse {
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[async_trait]
impl AIAgent for OpenAIAgent {
    /// Retorna o nome do agente: "openai"
    fn name(&self) -> &str {
        "openai"
    }

    /// Processa uma requisição enviando-a para a API OpenAI.
    ///
    /// # Parâmetros esperados no payload
    /// * `user_prompt` - O prompt do usuário (obrigatório)
    ///
    /// # Formato da resposta
    /// A resposta terá o comando "openai_response" e o payload conterá:
    /// * `answer` - O texto da resposta gerada pelo modelo
    ///
    /// # Erros
    /// * Retorna `MCPError::InternalAgentError` se:
    ///   - O campo `user_prompt` estiver ausente
    ///   - Houver falha na comunicação com a API
    ///   - A resposta da API não puder ser processada
    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        // Extrair o prompt do usuário do payload
        let user_prompt = message
            .payload
            .get("user_prompt")
            .and_then(Value::as_str)
            .ok_or_else(|| MCPError::InternalAgentError("Missing user_prompt".to_string()))?;

        // Construir o corpo da requisição
        let request_body = OpenAIChatRequest {
            model: self.model.clone(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
        };

        // Preparar os headers
        let headers = vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", self.api_key),
            ),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        // Enviar a requisição para a API OpenAI
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

        // Verificar o status da resposta
        if !response.status().is_success() {
            return Err(MCPError::InternalAgentError(format!(
                "OpenAI API retornou status {}",
                response.status()
            )));
        }

        // Deserializar a resposta
        let resp_json = response
            .json::<OpenAIChatResponse>()
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        // Extrair o texto da resposta
        let answer_text = resp_json
            .choices
            .get(0)
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| MCPError::InternalAgentError("No response choices".to_string()))?;

        // Retornar a resposta formatada como MCPMessage
        Ok(MCPMessage::new(
            "openai_response",
            json!({ "answer": answer_text }),
        ))
    }
}

/// Função auxiliar para criar um agente OpenAI com configurações do ambiente.
///
/// Esta função facilita a criação de uma instância do agente OpenAI, obtendo
/// a chave de API da variável de ambiente `OPENAI_API_KEY`.
///
/// # Argumentos
/// * `http_client` - Cliente HTTP opcional. Se None, será criado um novo.
///
/// # Retorno
/// Uma nova instância de `OpenAIAgent` configurada.
///
/// # Exemplo
///
/// ```
/// use mcprs::agent_openai::create_openai_agent;
///
/// // Configurar a variável de ambiente primeiro
/// std::env::set_var("OPENAI_API_KEY", "sua-chave-api");
///
/// // Criar o agente
/// let agent = create_openai_agent(None);
/// ```
pub fn create_openai_agent(http_client: Option<Box<dyn HttpClient>>) -> OpenAIAgent {
    let client = http_client.unwrap_or_else(|| Box::new(crate::testing::ReqwestClient::new()));

    OpenAIAgent::new(
        env::var("OPENAI_API_KEY").unwrap_or_else(|_| "SUA_KEY_AQUI".to_string()),
        "gpt-3.5-turbo".to_string(),
        client,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockHttpClient;
    use mockall::predicate;
    use serde_json::json;

    fn create_mock_response(body: serde_json::Value) -> reqwest::Response {
        reqwest::Response::from(
            http::Response::builder()
                .status(200)
                .body(body.to_string())
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_openai_agent_missing_prompt() {
        let mock_client = MockHttpClient::new();
        let agent = OpenAIAgent::new(
            "test_key".to_string(),
            "gpt-3.5-turbo".to_string(),
            Box::new(mock_client),
        );

        // Payload sem o campo user_prompt
        let message = MCPMessage::new("openai:chat", json!({}));
        let result = agent.process_request(message).await;

        assert!(
            matches!(result, Err(MCPError::InternalAgentError(e)) if e.contains("Missing user_prompt"))
        );
    }

    #[tokio::test]
    async fn test_openai_agent_successful_request() {
        let mut mock_client = MockHttpClient::new();

        mock_client
            .expect_post()
            .with(
                predicate::eq("https://api.openai.com/v1/chat/completions".to_string()),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .return_once(move |_, _, _| {
                Ok(create_mock_response(json!({
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": "Rust é uma linguagem de programação focada em segurança, desempenho e concorrência."
                        }
                    }]
                })))
            });

        let agent = OpenAIAgent::new(
            "test_key".to_string(),
            "gpt-3.5-turbo".to_string(),
            Box::new(mock_client),
        );

        let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "O que é Rust?" }));
        let result = agent.process_request(message).await.unwrap();

        assert_eq!(result.command, "openai_response");
        assert_eq!(
            result.payload["answer"],
            "Rust é uma linguagem de programação focada em segurança, desempenho e concorrência."
        );
    }
}
