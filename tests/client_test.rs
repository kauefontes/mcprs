use mcprs::client::create_mcp_message_for_agent;
use serde_json::json;

#[test]
fn test_create_mcp_message() {
    let msg = create_mcp_message_for_agent("dummy", "test", json!({"some": "payload"}));
    assert_eq!(msg.command, "dummy:test");
    assert_eq!(msg.payload, json!({"some": "payload"}));
    assert_eq!(msg.magic, "MCP0");
    assert_eq!(msg.version, 1);
}

// Teste local simples de send_mcp_request com um endpoint provavelmente inexistente.
/*
#[tokio::test]
async fn test_send_mcp_request_failure() {
    let msg = MCPMessage::new("dummy:fail", json!({}));
    let result = send_mcp_request("http://127.0.0.1:9999/mcp", &msg).await;
    assert!(matches!(result, Err(MCPClientError::NetworkError(_))));
}
*/
