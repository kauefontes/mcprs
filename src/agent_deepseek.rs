//! # Agente para integração com a API DeepSeek
//!
//! Este módulo implementa um agente que se comunica com a API do DeepSeek,
//! permitindo enviar prompts para os modelos de linguagem oferecidos pela plataforma.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::agent::{AgentRegistry, MCPMessage};
//! use mcprs::agent_deepseek::create_deepseek_agent;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configurar variável de ambiente (ou diretamente na criação do agente)
//! std::env::set_var("DEEPSEEK_API_KEY", "sua-chave-aqui");
//!
//! // Criar e registrar o agente DeepSeek
//! let mut registry = AgentRegistry::new();
//! registry.register_agent(Box::new(create_deepseek_agent(None)));
//!
//! // Criar uma mensagem para o modelo do DeepSeek
//! let message = MCPMessage::new(
//!     "deepseek:chat",
//!     json!({
//!         "user_prompt": "O que é computação quântica?"
//!     })
//! );
//!
//! // Processar a mensagem
//! let response = registry.process(message).await?;
//! println!("Resposta: {}", response.payload["answer"]);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

use crate::agent::{AIAgent, MCPError, MCPMessage};
use crate::testing::HttpClient;

/// Agente para comunicação com a API DeepSeek.
///
/// Este agente implementa a trait `AIAgent` e se conecta aos endpoints do
/// DeepSeek para enviar prompts e obter respostas dos modelos disponíveis.
pub struct DeepSeekAgent {
    /// Chave de API do DeepSeek
    pub api_key: String,

    /// URL base do endpoint DeepSeek (exemplo: https://api.deepseek.ai)
    pub endpoint: String,

    /// Nome do modelo a ser usado
    pub model: String,

    /// Cliente HTTP para fazer as requisições
    http_client: Box<dyn HttpClient>,
}

impl DeepSeekAgent {
    /// Cria uma nova instância do agente DeepSeek.
    ///
    /// # Argumentos
    /// * `api_key` - Chave de API do DeepSeek
    /// * `endpoint` - URL base do endpoint DeepSeek
    /// * `model` - Nome do modelo a ser usado
    /// * `http_client` - Cliente HTTP para fazer as requisições
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::agent_deepseek::DeepSeekAgent;
    /// use mcprs::testing::ReqwestClient;
    ///
    /// let agent = DeepSeekAgent::new(
    ///     "sua-chave-api".to_string(),
    ///     "https://api.deepseek.ai".to_string(),
    ///     "deepseek-chat".to_string(),
    ///     Box::new(ReqwestClient::new())
    /// );
    /// ```
    pub fn new(
        api_key: String,
        endpoint: String,
        model: String,
        http_client: Box<dyn HttpClient>,
    ) -> Self {
        Self {
            api_key,
            endpoint,
            model,
            http_client,
        }
    }
}

/// Estrutura para o corpo da requisição à API DeepSeek
#[derive(Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

/// Estrutura para uma mensagem na requisição à API DeepSeek
#[derive(Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

/// Estrutura para a resposta da API DeepSeek
#[derive(Deserialize)]
struct DeepSeekResponse {
    id: String,
    choices: Vec<DeepSeekChoice>,
}

/// Estrutura para um item de escolha na resposta da API DeepSeek
#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessageResponse,
    finish_reason: String,
}

/// Estrutura para a mensagem dentro de um item de escolha na resposta
#[derive(Deserialize)]
struct DeepSeekMessageResponse {
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[async_trait]
impl AIAgent for DeepSeekAgent {
    /// Retorna o nome do agente: "deepseek"
    fn name(&self) -> &str {
        "deepseek"
    }

    /// Processa uma requisição enviando-a para a API DeepSeek.
    ///
    /// # Parâmetros esperados no payload
    /// * `user_prompt` - O prompt do usuário (obrigatório)
    /// * `temperature` - Temperatura para geração (opcional)
    /// * `max_tokens` - Limite de tokens na resposta (opcional)
    ///
    /// # Formato da resposta
    /// A resposta terá o comando "deepseek_response" e o payload conterá:
    /// * `answer` - O texto da resposta gerada pelo modelo
    /// * `id` - O ID da resposta gerada pela API
    /// * `finish_reason` - A razão de término da geração (stop, length, etc.)
    ///
    /// # Erros
    /// * Retorna `MCPError::InternalAgentError` se:
    ///   - O campo `user_prompt` estiver ausente
    ///   - Houver falha na comunicação com a API
    ///   - A resposta da API não puder ser processada
    async fn process_request(&self, message: MCPMessage) -> Result<MCPMessage, MCPError> {
        // Extrair o prompt de usuário do payload
        let user_prompt = message
            .payload
            .get("user_prompt".to_owned())
            .and_then(Value::as_str)
            .ok_or_else(|| MCPError::InternalAgentError("Missing user_prompt".to_string()))?;

        // Estruturar a requisição para DeepSeek
        let request_body = DeepSeekRequest {
            model: self.model.clone(),
            messages: vec![DeepSeekMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
            temperature: message
                .payload
                .get("temperature".to_owned())
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            max_tokens: message
                .payload
                .get("max_tokens".to_owned())
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        };

        // Configurar headers
        let headers = vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", self.api_key),
            ),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        // Enviar requisição para a API DeepSeek
        let response = self
            .http_client
            .post(
                format!("{}/v1/chat/completions", self.endpoint),
                serde_json::to_vec(&request_body)
                    .map_err(|e| MCPError::InternalAgentError(e.to_string()))?,
                headers,
            )
            .await
            .map_err(|e| MCPError::InternalAgentError(e.to_string()))?;

        // Validar status da resposta
        if !response.status().is_success() {
            return Err(MCPError::InternalAgentError(format!(
                "DeepSeek API retornou status {}",
                response.status()
            )));
        }

        // Desserializar e processar a resposta
        let resp_json = response
            .json::<DeepSeekResponse>()
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
            "deepseek_response",
            json!({
                "answer": answer_text,
                "id": resp_json.id,
                "finish_reason": resp_json.choices.get(0).map(|c| &c.finish_reason).unwrap_or(&"unknown".to_string())
            }),
        ))
    }
}

/// Função auxiliar para criar um agente DeepSeek com configurações do ambiente.
///
/// Esta função facilita a criação de uma instância do agente DeepSeek, obtendo
/// as configurações das variáveis de ambiente:
/// - `DEEPSEEK_API_KEY` - Chave de API
/// - `DEEPSEEK_ENDPOINT` - URL base do endpoint (padrão: https://api.deepseek.ai)
/// - `DEEPSEEK_MODEL` - Nome do modelo (padrão: deepseek-chat)
///
/// # Argumentos
/// * `http_client` - Cliente HTTP opcional. Se None, será criado um novo.
///
/// # Retorno
/// Uma nova instância de `DeepSeekAgent` configurada.
///
/// # Exemplo
///
/// ```
/// use mcprs::agent_deepseek::create_deepseek_agent;
///
/// // Configurar a variável de ambiente primeiro
/// std::env::set_var("DEEPSEEK_API_KEY", "sua-chave-api");
///
/// // Criar o agente
/// let agent = create_deepseek_agent(None);
/// ```
pub fn create_deepseek_agent(http_client: Option<Box<dyn HttpClient>>) -> DeepSeekAgent {
    let client = http_client.unwrap_or_else(|| Box::new(crate::testing::ReqwestClient::new()));

    DeepSeekAgent::new(
        env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "SUA_DEEPSEEK_KEY".to_string()),
        env::var("DEEPSEEK_ENDPOINT").unwrap_or_else(|_| "https://api.deepseek.ai".to_string()),
        env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()),
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
    async fn test_deepseek_agent_missing_prompt() {
        let mock_client = MockHttpClient::new();
        let agent = DeepSeekAgent::new(
            "test_key".to_string(),
            "https://api.test.deepseek.ai".to_string(),
            "test-model".to_string(),
            Box::new(mock_client),
        );

        // Payload sem o campo user_prompt
        let message = MCPMessage::new("deepseek:chat", json!({ "wrong_field": "value" }));
        let result = agent.process_request(message).await;

        assert!(
            matches!(result, Err(MCPError::InternalAgentError(e)) if e.contains("Missing user_prompt"))
        );
    }

    #[tokio::test]
    async fn test_deepseek_agent_successful_request() {
        let mut mock_client = MockHttpClient::new();

        mock_client
            .expect_post()
            .with(
                predicate::function(|url: &String| url.contains("chat/completions")),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .return_once(move |_, _, _| {
                Ok(create_mock_response(json!({
                    "id": "ds-1234567890",
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": "Computação quântica é um paradigma de computação que utiliza fenômenos quânticos para realizar operações em dados."
                        },
                        "finish_reason": "stop"
                    }]
                })))
            });

        let agent = DeepSeekAgent::new(
            "test_key".to_string(),
            "https://api.test.deepseek.ai".to_string(),
            "test-model".to_string(),
            Box::new(mock_client),
        );

        let message = MCPMessage::new(
            "deepseek:chat",
            json!({ "user_prompt": "O que é computação quântica?" }),
        );

        let result = agent.process_request(message).await.unwrap();

        assert_eq!(result.command, "deepseek_response");
        assert_eq!(
            result.payload["answer"],
            "Computação quântica é um paradigma de computação que utiliza fenômenos quânticos para realizar operações em dados."
        );
        assert_eq!(result.payload["id"], "ds-1234567890");
        assert_eq!(result.payload["finish_reason"], "stop");
    }

    #[tokio::test]
    async fn test_deepseek_agent_with_parameters() {
        let mut mock_client = MockHttpClient::new();

        // Verificar se os parâmetros opcionais são passados corretamente
        mock_client
            .expect_post()
            .withf(|_, body, _| {
                if let Ok(parsed) = serde_json::from_slice::<Value>(body) {
                    return parsed["temperature"].as_f64().unwrap_or(0.0) == 0.7
                        && parsed["max_tokens"].as_u64().unwrap_or(0) == 100;
                }
                false
            })
            .return_once(move |_, _, _| {
                Ok(create_mock_response(json!({
                    "id": "ds-test-params",
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": "Resposta de teste com parâmetros"
                        },
                        "finish_reason": "stop"
                    }]
                })))
            });

        let agent = DeepSeekAgent::new(
            "test_key".to_string(),
            "https://api.test.deepseek.ai".to_string(),
            "test-model".to_string(),
            Box::new(mock_client),
        );

        let message = MCPMessage::new(
            "deepseek:chat",
            json!({
                "user_prompt": "Teste com parâmetros",
                "temperature": 0.7,
                "max_tokens": 100
            }),
        );

        let result = agent.process_request(message).await.unwrap();
        assert_eq!(result.payload["answer"], "Resposta de teste com parâmetros");
    }
}
