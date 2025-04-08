use mcprs::agent::{AIAgent, MCPError, MCPMessage};
use mcprs::testing::MockHttpClient;
use mockall::predicate;
use serde_json::json;

// Helper para criar uma resposta mockada
fn create_mock_response(body: serde_json::Value) -> reqwest::Response {
    reqwest::Response::from(
        http::Response::builder()
            .status(200)
            .body(body.to_string())
            .unwrap(),
    )
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
                        "content": "Mock response"
                    }
                }]
            })))
        });

    let agent = mcprs::agent_openai::create_openai_agent(Some(Box::new(mock_client)));
    let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "Test prompt" }));

    let result = agent.process_request(message).await.unwrap();
    assert_eq!(result.command, "openai_response");
    assert_eq!(result.payload, json!({ "answer": "Mock response" }));
}

#[tokio::test]
async fn test_openai_agent_network_error() {
    let mut mock_client = MockHttpClient::new();

    mock_client.expect_post().return_once(|_, _, _| {
        Ok(reqwest::Response::from(
            http::Response::builder()
                .status(500)
                .body("Internal Server Error")
                .unwrap(),
        ))
    });

    let agent = mcprs::agent_openai::create_openai_agent(Some(Box::new(mock_client)));
    let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "Test prompt" }));

    let result = agent.process_request(message).await;
    assert!(matches!(result, Err(MCPError::InternalAgentError(_))));
}

#[tokio::test]
async fn test_openai_agent_api_error() {
    let mut mock_client = MockHttpClient::new();

    mock_client.expect_post().return_once(|_, _, _| {
        Ok(reqwest::Response::from(
            http::Response::builder()
                .status(400)
                .body("Bad Request")
                .unwrap(),
        ))
    });

    let agent = mcprs::agent_openai::create_openai_agent(Some(Box::new(mock_client)));
    let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "Test prompt" }));

    let result = agent.process_request(message).await;
    assert!(matches!(result, Err(MCPError::InternalAgentError(_))));
}

#[tokio::test]
async fn test_openai_agent_create_with_environment() {
    // Definir variáveis de ambiente temporariamente
    std::env::set_var("OPENAI_API_KEY", "test-key-from-env");

    // Criar um mock que verifica se a chave correta está sendo usada
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_post()
        .withf(|_, _, headers| {
            // Procurar o header Authorization e verificar se contém a chave definida
            headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer test-key-from-env")
        })
        .return_once(|_, _, _| {
            Ok(create_mock_response(json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Resposta de teste"
                    }
                }]
            })))
        });

    let agent = mcprs::agent_openai::create_openai_agent(Some(Box::new(mock_client)));
    let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "Teste com env var" }));

    let result = agent.process_request(message).await.unwrap();
    assert_eq!(result.payload["answer"], "Resposta de teste");

    // Limpar a variável de ambiente após o teste
    std::env::remove_var("OPENAI_API_KEY");
}

#[tokio::test]
async fn test_openai_agent_custom_model() {
    let mut mock_client = MockHttpClient::new();

    // Verificar se o modelo correto está sendo enviado na requisição
    mock_client
        .expect_post()
        .withf(|_, body, _| {
            // Verificar se o corpo da requisição contém o modelo correto
            let parsed: serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
            parsed["model"] == "gpt-4"
        })
        .return_once(|_, _, _| {
            Ok(create_mock_response(json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Resposta do GPT-4"
                    }
                }]
            })))
        });

    // Criar agente diretamente com modelo personalizado
    let agent = mcprs::agent_openai::OpenAIAgent::new(
        "chave-teste".to_string(),
        "gpt-4".to_string(),
        Box::new(mock_client),
    );

    let message = MCPMessage::new("openai:chat", json!({ "user_prompt": "Teste com GPT-4" }));
    let result = agent.process_request(message).await.unwrap();

    assert_eq!(result.payload["answer"], "Resposta do GPT-4");
}

#[tokio::test]
async fn test_openai_agent_missing_prompt() {
    let mock_client = MockHttpClient::new();
    let agent = mcprs::agent_openai::create_openai_agent(Some(Box::new(mock_client)));

    // Payload sem o campo user_prompt
    let message = MCPMessage::new(
        "openai:chat",
        json!({
            "temperature": 0.7,
            "max_tokens": 100
        }),
    );

    let result = agent.process_request(message).await;
    assert!(
        matches!(result, Err(MCPError::InternalAgentError(e)) if e.contains("Missing user_prompt"))
    );
}
