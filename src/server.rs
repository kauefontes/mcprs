use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber;

use crate::agent::{AgentRegistry, MCPError, MCPMessage};

/// Estrutura para representar uma resposta de erro em JSON.
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

/// Converte um MCPError em uma resposta HTTP.
impl IntoResponse for MCPError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

/// Inicia e executa o servidor HTTP MCP.
/// O `registry` é compartilhado entre as requisições.
/// Em produção, você provavelmente adicionaria autenticação, TLS etc.
pub async fn run_http_server(registry: AgentRegistry, addr: SocketAddr) {
    // Inicializa o logging.
    tracing_subscriber::fmt::init();

    let shared_registry = Arc::new(RwLock::new(registry));

    // Configura o roteador com a rota /mcp para requisições POST.
    let app = Router::new().route(
        "/mcp",
        post({
            let registry = Arc::clone(&shared_registry);
            move |payload| handle_mcp(payload, registry)
        }),
    );

    info!("Servidor MCP rodando em {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handler para a rota /mcp.
/// Recebe uma MCPMessage, valida e processa a mensagem.
async fn handle_mcp(
    Json(payload): Json<MCPMessage>,
    registry: Arc<RwLock<AgentRegistry>>,
) -> Result<Json<MCPMessage>, MCPError> {
    // Validação do campo magic.
    if payload.magic != "MCP0" {
        error!("Magic inválido: {}", payload.magic);
        return Ok(Json(MCPMessage::new(
            "error",
            json!({"message": "Magic inválido"}),
        )));
    }

    // Processa a mensagem utilizando o registro de agentes.
    let response = {
        let reg = registry.read().await;
        reg.process(payload).await?
    };

    Ok(Json(response))
}
