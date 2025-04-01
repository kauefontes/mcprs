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
