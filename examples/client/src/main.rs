use mcprs::client::{create_mcp_message_for_agent, send_mcp_request};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Definir URL do servidor e token de autenticação
    let server_url = "http://localhost:3000/mcp";
    let auth_token = "seu-token-de-api-aqui";

    // Criar uma mensagem para o agente OpenAI
    let message = create_mcp_message_for_agent(
        "openai",
        "chat",
        json!({
            "user_prompt": "Explique a linguagem Rust para iniciantes",
            "conversation_id": "opcional-para-manter-contexto"
        }),
    );

    // Configurar o cliente com autenticação
    let client = reqwest::Client::new();
    let response = client
        .post(server_url)
        .bearer_auth(auth_token)
        .json(&message)
        .send()
        .await?;

    if response.status().is_success() {
        let mcp_response = response.json::<mcprs::MCPMessage>().await?;
        println!("Resposta: {}", mcp_response.payload["answer"]);
    } else {
        println!("Erro: {} - {}", response.status(), response.text().await?);
    }

    Ok(())
}
