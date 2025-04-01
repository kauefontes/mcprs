use mcprs::agent::{AgentRegistry, DummyAgent, MCPError, MCPMessage};
use serde_json::json;

#[tokio::test]
async fn test_registry_register_and_process_dummy() {
    let mut registry = AgentRegistry::new();
    registry.register_agent(Box::new(DummyAgent {
        api_key: "dummy_key".to_string(),
    }));

    let msg = MCPMessage::new("dummy:echo", json!({"hello": "world"}));
    let result = registry.process(msg.clone()).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.command, "dummy_response");
    assert_eq!(response.payload, msg.payload);
}

#[tokio::test]
async fn test_registry_agent_not_found() {
    let registry = AgentRegistry::new(); // vazio

    let msg = MCPMessage::new("nope:action", json!({"hello": "world"}));
    let result = registry.process(msg).await;

    assert!(result.is_err());
    if let Err(MCPError::AgentNotRegistered(name)) = result {
        assert_eq!(name.as_str(), "nope");
    } else {
        panic!("Esperava AgentNotRegistered, mas não veio.");
    }
}

#[tokio::test]
async fn test_registry_invalid_command_format() {
    // Falta o ':', então deve disparar MCPError::InvalidCommandFormat
    let registry = AgentRegistry::new();

    let msg = MCPMessage::new("invalido", json!({}));
    let result = registry.process(msg).await;
    assert!(matches!(result, Err(MCPError::InvalidCommandFormat)));
}
