use mcprs::agent::{AgentRegistry, DummyAgent, MCPMessage};
use mcprs::agent_deepseek::create_deepseek_agent;
use mcprs::agent_openai::create_openai_agent;
use mcprs::client::{create_mcp_message_for_agent, send_mcp_request};
use mcprs::server::run_http_server; // Mantém função original para compatibilidade
use serde_json::json;
use std::net::SocketAddr;
use tokio::task;

#[tokio::main]
async fn main() {
    // Cria e configura o AgentRegistry
    let mut registry = AgentRegistry::new();

    // DummyAgent (apenas ecoa payload)
    registry.register_agent(Box::new(DummyAgent {
        api_key: "dummy_key".to_string(),
    }));

    // OpenAIAgent (chaves via env ou definidas manualmente)
    let openai_agent = create_openai_agent(None);
    registry.register_agent(Box::new(openai_agent));

    // DeepSeekAgent
    let deepseek_agent = create_deepseek_agent(None);
    registry.register_agent(Box::new(deepseek_agent));

    // Endereço do servidor
    let addr: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    let server_task = task::spawn(run_http_server(registry, addr));
    println!("Servidor MCP ouvindo em {}", addr);

    // Aguardar um instante
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Exemplo: criar uma mensagem para o dummy
    let msg_dummy = create_mcp_message_for_agent("dummy", "demo", json!({"exemplo": "123"}));
    match send_mcp_request("http://127.0.0.1:4001/mcp", &msg_dummy).await {
        Ok(resp) => println!("DummyAgent respondeu: {:?}", resp),
        Err(e) => eprintln!("Erro chamando DummyAgent: {}", e),
    }

    // Exemplo: criar uma mensagem para openai
    // (ATENÇÃO: sem mock, isso vai tentar chamar a API real do OpenAI!)
    let msg_openai = create_mcp_message_for_agent(
        "openai",
        "chat",
        json!({
            "user_prompt": "Fale uma curiosidade sobre Rust"
        }),
    );
    match send_mcp_request("http://127.0.0.1:4001/mcp", &msg_openai).await {
        Ok(resp) => println!("OpenAIAgent respondeu: {:?}", resp),
        Err(e) => eprintln!("Erro chamando OpenAIAgent: {}", e),
    }

    // Exemplo: criar uma mensagem para deepseek
    let msg_deepseek = create_mcp_message_for_agent(
        "deepseek",
        "ask",
        json!({
            "question": "Quem descobriu o Brasil?"
        }),
    );
    match send_mcp_request("http://127.0.0.1:4001/mcp", &msg_deepseek).await {
        Ok(resp) => println!("DeepSeekAgent respondeu: {:?}", resp),
        Err(e) => eprintln!("Erro chamando DeepSeekAgent: {}", e),
    }

    // Para não travar este exemplo, finalizamos
    server_task.abort();
    println!("Encerrando demonstrate_agents example...");
}
