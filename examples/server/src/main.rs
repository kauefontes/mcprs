use mcprs::agent::AgentRegistry;
use mcprs::agent_deepseek::create_deepseek_agent;
use mcprs::agent_openai::create_openai_agent;
use mcprs::auth::AuthConfig;
use mcprs::conversation::ConversationManager;
use mcprs::server::run_http_server_with_auth; // Alterado para usar a nova função
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Criar o registro de agentes
    let mut registry = AgentRegistry::new();

    // Registrar os agentes - OpenAI e DeepSeek
    registry.register_agent(Box::new(create_openai_agent(None)));
    registry.register_agent(Box::new(create_deepseek_agent(None)));

    // Configurar autenticação
    let auth_config = AuthConfig::new();
    auth_config.add_token("seu-token-de-api-aqui".to_string());

    // Configurar gerenciador de conversas (manter histórico por 24 horas)
    let conversation_manager = ConversationManager::new(24);

    // Agendar limpeza periódica de conversas antigas
    let conversation_manager_clone = conversation_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let cleaned = conversation_manager_clone.cleanup_old_conversations();
            println!("Limpeza de conversas: {} removidas", cleaned);
        }
    });

    // Iniciar o servidor na porta 3000
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    run_http_server_with_auth(registry, auth_config, conversation_manager, addr).await;
}
