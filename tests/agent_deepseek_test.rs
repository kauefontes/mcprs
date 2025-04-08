use mcprs::agent::{AIAgent, MCPError, MCPMessage};
use mcprs::agent_deepseek::DeepSeekAgent;
use mcprs::testing::MockHttpClient;
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
async fn test_deepseek_agent_successful_request() {
    let mut mock_client = MockHttpClient::new();

    mock_client
        .expect_post()
        .with(
            // Usar predicate::function para criar um predicado que funcione com String
            predicate::function(|url: &String| url.contains("chat/completions")),
            predicate::always(),
            predicate::always(),
        )
        .times(1)
        .return_once(move |_, _, _| {
            Ok(create_mock_response(json!({
                "id": "resp-1234567890",
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Esta é uma resposta de teste do DeepSeek"
                    },
                    "finish_reason": "stop"
                }]
            })))
        });

    let agent = DeepSeekAgent::new(
        "test-api-key".to_string(),
        "https://api.deepseek.ai".to_string(),
        "deepseek-chat".to_string(),
        Box::new(mock_client),
    );

    let message = MCPMessage::new(
        "deepseek:chat",
        json!({ "user_prompt": "Teste da API DeepSeek" }),
    );

    let result = agent.process_request(message).await.unwrap();
    assert_eq!(result.command, "deepseek_response");
    assert_eq!(
        result.payload["answer"],
        "Esta é uma resposta de teste do DeepSeek"
    );
    assert_eq!(result.payload["id"], "resp-1234567890");
    assert_eq!(result.payload["finish_reason"], "stop");
}

#[tokio::test]
async fn test_deepseek_agent_missing_prompt() {
    let mock_client = MockHttpClient::new();

    let agent = DeepSeekAgent::new(
        "test-api-key".to_string(),
        "https://api.deepseek.ai".to_string(),
        "deepseek-chat".to_string(),
        Box::new(mock_client),
    );

    let message = MCPMessage::new(
        "deepseek:chat",
        json!({ "wrong_field": "Não tem user_prompt" }),
    );

    let result = agent.process_request(message).await;
    assert!(matches!(result, Err(MCPError::InternalAgentError(_))));
}

#[tokio::test]
async fn test_deepseek_agent_with_parameters() {
    let mut mock_client = MockHttpClient::new();

    // Verificar se os parâmetros são passados corretamente
    mock_client
        .expect_post()
        .withf(|_, body, _| {
            if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(body) {
                return parsed["temperature"].as_f64().unwrap_or(0.0) == 0.7
                    && parsed["max_tokens"].as_u64().unwrap_or(0) == 100;
            }
            false
        })
        .return_once(|_, _, _| {
            Ok(create_mock_response(json!({
                "id": "ds-params-test",
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Resposta com parâmetros customizados"
                    },
                    "finish_reason": "stop"
                }]
            })))
        });

    let agent = DeepSeekAgent::new(
        "test-api-key".to_string(),
        "https://api.deepseek.ai".to_string(),
        "deepseek-chat".to_string(),
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
    assert_eq!(
        result.payload["answer"],
        "Resposta com parâmetros customizados"
    );
}

#[tokio::test]
async fn test_deepseek_agent_api_error() {
    let mut mock_client = MockHttpClient::new();

    // Simular erro de API
    mock_client.expect_post().return_once(|_, _, _| {
        Ok(reqwest::Response::from(
            http::Response::builder()
                .status(401)
                .body("Unauthorized")
                .unwrap(),
        ))
    });

    let agent = DeepSeekAgent::new(
        "invalid-key".to_string(),
        "https://api.deepseek.ai".to_string(),
        "deepseek-chat".to_string(),
        Box::new(mock_client),
    );

    let message = MCPMessage::new("deepseek:chat", json!({ "user_prompt": "Teste com erro" }));

    let result = agent.process_request(message).await;
    assert!(matches!(result, Err(MCPError::InternalAgentError(e)) if e.contains("status 401")));
}

#[tokio::test]
async fn test_deepseek_agent_environment_vars() {
    // Definir variáveis de ambiente temporariamente
    std::env::set_var("DEEPSEEK_API_KEY", "env-test-key");
    std::env::set_var("DEEPSEEK_ENDPOINT", "https://custom.deepseek.test");
    std::env::set_var("DEEPSEEK_MODEL", "deepseek-chat-plus");

    // Criar mock que verifica se os valores das variáveis de ambiente são usados
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_post()
        .withf(|url, body, headers| {
            // Verificar URL do endpoint
            let correct_url = url.starts_with("https://custom.deepseek.test");

            // Verificar modelo
            let model_ok = if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(body) {
                parsed["model"] == "deepseek-chat-plus"
            } else {
                false
            };

            // Verificar API key
            let key_ok = headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer env-test-key");

            correct_url && model_ok && key_ok
        })
        .return_once(|_, _, _| {
            Ok(create_mock_response(json!({
                "id": "env-test-response",
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Resposta das variáveis de ambiente"
                    },
                    "finish_reason": "stop"
                }]
            })))
        });

    // Criar agente usando factory que lê variáveis de ambiente
    let agent = mcprs::agent_deepseek::create_deepseek_agent(Some(Box::new(mock_client)));

    let message = MCPMessage::new(
        "deepseek:chat",
        json!({ "user_prompt": "Teste de variáveis de ambiente" }),
    );

    let result = agent.process_request(message).await.unwrap();
    assert_eq!(
        result.payload["answer"],
        "Resposta das variáveis de ambiente"
    );

    // Limpar variáveis de ambiente
    std::env::remove_var("DEEPSEEK_API_KEY");
    std::env::remove_var("DEEPSEEK_ENDPOINT");
    std::env::remove_var("DEEPSEEK_MODEL");
}
