use mcprs::agent::AIAgent;
use mcprs::agent_deepseek::DeepSeekAgent;

#[tokio::test]
async fn test_deepseek_agent_basics() {
    let agent = DeepSeekAgent {
        api_key: "FAKE_DEEPSEEK_KEY".to_string(),
        endpoint: "https://api.deepseek.ai".to_string(),
    };

    assert_eq!(agent.name(), "deepseek");

    // Mesmo caso do OpenAI se não mockarmos as chamadas HTTP, este teste tentaria chamar a rede real.

    /*
    let message = MCPMessage::new("deepseek:consulta", json!({"question": "Quem descobriu o Brasil?"}));
    let result = agent.process_request(message).await;
    // Esperar erro sem mock
    */
}
