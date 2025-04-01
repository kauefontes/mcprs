use mcprs::agent::{AgentRegistry, DummyAgent, MCPMessage};
use mcprs::client::send_mcp_request;
use mcprs::server::run_http_server;
use serde_json::json;
use std::net::SocketAddr;
use tokio::task;

#[tokio::test]
async fn test_dummy_integration() {
    // Cria um registro e registra o DummyAgent.
    let mut registry = AgentRegistry::new();
    registry.register_agent(Box::new(DummyAgent {
        api_key: "some-dummy-key".to_string(),
    }));

    // Define o endereço para o teste. Em CI, verifique se a porta está livre.
    let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();

    // Sobe o servidor em uma task separada.
    let server_task = task::spawn(run_http_server(registry, addr));

    // Aguarda um instante para o servidor iniciar de fato.
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Cria uma mensagem MCP de teste direcionada ao agente dummy.
    let msg = MCPMessage::new("dummy:test_integration", json!({"test": "value"}));

    // Faz requisição via client do crate
    let resp = send_mcp_request("http://127.0.0.1:4000/mcp", &msg)
        .await
        .expect("Falha no envio da requisição MCP");

    // O DummyAgent deve responder com "dummy_response" e ecoar o payload.
    assert_eq!(resp.command, "dummy_response");
    assert_eq!(resp.payload, json!({"test": "value"}));

    // Aborta o servidor neste teste
    server_task.abort();
}
