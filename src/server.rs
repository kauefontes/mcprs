//! # Módulo de Servidor MCP
//!
//! Este módulo implementa um servidor HTTP para receber e processar mensagens MCP.
//! Ele fornece duas versões do servidor: uma básica e uma avançada com autenticação
//! e gerenciamento de conversas.
//!
//! ## Exemplo de Uso Básico
//!
//! ```rust,no_run
//! use mcprs::agent::AgentRegistry;
//! use mcprs::agent_openai::create_openai_agent;
//! use mcprs::server::run_http_server;
//! use std::net::SocketAddr;
//!
//! # async fn example() {
//! // Configurar variável de ambiente
//! std::env::set_var("OPENAI_API_KEY", "sua-chave-aqui");
//!
//! // Criar e configurar registro de agentes
//! let mut registry = AgentRegistry::new();
//! registry.register_agent(Box::new(create_openai_agent(None)));
//!
//! // Iniciar o servidor HTTP
//! let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//! run_http_server(registry, addr).await;
//! # }
//! ```
//!
//! ## Exemplo de Uso Avançado
//!
//! ```rust,no_run
//! use mcprs::agent::AgentRegistry;
//! use mcprs::agent_openai::create_openai_agent;
//! use mcprs::auth::AuthConfig;
//! use mcprs::conversation::ConversationManager;
//! use mcprs::server::run_http_server_with_auth;
//! use std::net::SocketAddr;
//!
//! # async fn example() {
//! // Configurar os componentes
//! let mut registry = AgentRegistry::new();
//! registry.register_agent(Box::new(create_openai_agent(None)));
//!
//! let auth_config = AuthConfig::new();
//! auth_config.add_token("token-secreto".to_string());
//!
//! let conversation_manager = ConversationManager::new(24);
//!
//! // Iniciar o servidor avançado
//! let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//! run_http_server_with_auth(registry, auth_config, conversation_manager, addr).await;
//! # }
//! ```

use axum::{
    extract::Json,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Extension, Router,
};
use futures::Stream;
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};
use tracing_subscriber;

use crate::agent::{AgentRegistry, MCPError, MCPMessage};
use crate::auth::AuthConfig;
use crate::conversation::ConversationManager;

/// Estado compartilhado da aplicação no servidor.
///
/// Esta estrutura mantém referências a todos os componentes principais
/// do servidor MCP e é compartilhada entre manipuladores de requisições.
#[derive(Clone)]
pub struct AppState {
    /// Registro de agentes para roteamento de mensagens
    registry: Arc<RwLock<AgentRegistry>>,

    /// Configuração de autenticação (opcional)
    #[allow(dead_code)]
    auth_config: Option<AuthConfig>,

    /// Gerenciador de conversas (opcional)
    conversation_manager: Option<Arc<ConversationManager>>,
}

/// Estrutura para representar uma resposta de erro em JSON.
#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorResponse {
    /// Mensagem de erro
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

/// Inicia e executa o servidor HTTP MCP básico.
///
/// Esta é a versão mais simples do servidor, sem autenticação ou
/// gerenciamento de conversas. Útil para testes e integração inicial.
///
/// # Argumentos
/// * `registry` - O registro de agentes para processar mensagens
/// * `addr` - O endereço e porta onde o servidor deve escutar
///
/// # Exemplo
///
/// ```rust,no_run
/// use mcprs::agent::AgentRegistry;
/// use mcprs::server::run_http_server;
/// use std::net::SocketAddr;
///
/// # async fn example() {
/// let registry = AgentRegistry::new();
/// let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
/// run_http_server(registry, addr).await;
/// # }
/// ```
pub async fn run_http_server(registry: AgentRegistry, addr: SocketAddr) {
    // Inicializa o logging.
    tracing_subscriber::fmt::init();

    // Criar AppState sem autenticação nem gerenciamento de conversa
    let app_state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        auth_config: None,
        conversation_manager: None,
    };

    // Configura o roteador com a rota /mcp para requisições POST.
    let app = Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/health", get(|| async { "OK" }))
        .with_state(app_state);

    info!("Servidor MCP rodando em {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Inicia e executa o servidor HTTP MCP avançado com autenticação e gestão de conversas.
///
/// Esta versão do servidor inclui:
/// - Autenticação via token Bearer
/// - Gerenciamento de histórico de conversas
/// - Suporte para streaming de respostas
/// - Endpoints adicionais para gerenciar conversações
///
/// # Argumentos
/// * `registry` - O registro de agentes para processar mensagens
/// * `auth_config` - Configuração de autenticação
/// * `conversation_manager` - Gerenciador de histórico de conversas
/// * `addr` - O endereço e porta onde o servidor deve escutar
///
/// # Exemplo
///
/// ```rust,no_run
/// use mcprs::agent::AgentRegistry;
/// use mcprs::auth::AuthConfig;
/// use mcprs::conversation::ConversationManager;
/// use mcprs::server::run_http_server_with_auth;
/// use std::net::SocketAddr;
///
/// # async fn example() {
/// let registry = AgentRegistry::new();
/// let auth_config = AuthConfig::new();
/// let conversation_manager = ConversationManager::new(24);
/// let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
///
/// run_http_server_with_auth(registry, auth_config, conversation_manager, addr).await;
/// # }
/// ```
pub async fn run_http_server_with_auth(
    registry: AgentRegistry,
    auth_config: AuthConfig,
    conversation_manager: ConversationManager,
    addr: SocketAddr,
) {
    // Inicializa o logging.
    tracing_subscriber::fmt::init();

    let app_state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        auth_config: Some(auth_config.clone()),
        conversation_manager: Some(Arc::new(conversation_manager)),
    };

    // Configura as rotas
    let app = Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/mcp/stream", get(handle_stream_mcp))
        .route("/conversation", post(create_conversation))
        .route("/conversation/:id", get(get_conversation))
        .route("/health", get(|| async { "OK" }))
        .with_state(app_state)
        .layer(Extension(auth_config));

    info!("Servidor MCP avançado rodando em {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handler para a rota /mcp.
///
/// Este handler recebe uma requisição POST com uma MCPMessage,
/// valida-a, e a encaminha para o agente apropriado.
///
/// # Argumentos
/// * `state` - O estado compartilhado da aplicação
/// * `payload` - A mensagem MCP recebida no corpo da requisição
///
/// # Retorna
/// * `Ok(Json<MCPMessage>)` - A resposta do agente
/// * `Err(MCPError)` - Se ocorrer um erro no processamento
async fn handle_mcp(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<MCPMessage>,
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
        let reg = state.registry.read().await;
        reg.process(payload).await?
    };

    Ok(Json(response))
}

/// Handler para o endpoint de streaming /mcp/stream.
///
/// Este handler é semelhante ao `handle_mcp`, mas retorna a resposta
/// como um stream de eventos (Server-Sent Events).
///
/// # Argumentos
/// * `state` - O estado compartilhado da aplicação
/// * `payload` - A mensagem MCP recebida no corpo da requisição
///
/// # Retorna
/// Um stream de eventos SSE com a resposta
async fn handle_stream_mcp(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<MCPMessage>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Inicia o processamento em uma task separada
    tokio::spawn(async move {
        // Validação do campo magic
        if payload.magic != "MCP0" {
            let _ = tx
                .send(Ok(Event::default().data("Error: Invalid magic")))
                .await;
            return;
        }

        // Processa a mensagem e envia resultados para o stream
        let reg = state.registry.read().await;
        match reg.process(payload).await {
            Ok(response) => {
                let _ = tx
                    .send(Ok(
                        Event::default().data(serde_json::to_string(&response).unwrap_or_default())
                    ))
                    .await;
            }
            Err(error) => {
                let _ = tx
                    .send(Ok(Event::default().data(format!("Error: {}", error))))
                    .await;
            }
        }
    });

    Sse::new(ReceiverStream::new(rx))
}

/// Endpoint para criar uma nova conversa.
///
/// # Argumentos
/// * `state` - O estado compartilhado da aplicação
///
/// # Retorna
/// * No sucesso: Status 201 Created com ID da conversa
/// * No erro: Status 500 Internal Server Error ou 501 Not Implemented
async fn create_conversation(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    if let Some(ref conversation_manager) = state.conversation_manager {
        match conversation_manager.create_conversation() {
            Ok(conversation) => (
                StatusCode::CREATED,
                Json(json!({
                    "conversation_id": conversation.id,
                    "created_at": conversation.created_at.elapsed().unwrap_or_default().as_secs()
                })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ),
        }
    } else {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "Gerenciamento de conversas não está habilitado" })),
        )
    }
}

/// Endpoint para obter uma conversa existente pelo ID.
///
/// # Argumentos
/// * `state` - O estado compartilhado da aplicação
/// * `id` - O ID da conversa a ser recuperada
///
/// # Retorna
/// * No sucesso: Status 200 OK com dados da conversa
/// * No erro: Status 404 Not Found ou 501 Not Implemented
async fn get_conversation(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some(ref conversation_manager) = state.conversation_manager {
        match conversation_manager.get_conversation(&id) {
            Some(conversation) => {
                let messages: Vec<_> = conversation
                    .messages
                    .iter()
                    .map(|msg| {
                        json!({
                            "role": msg.role,
                            "content": msg.content,
                            "timestamp": msg.timestamp.elapsed().unwrap_or_default().as_secs()
                        })
                    })
                    .collect();

                (
                    StatusCode::OK,
                    Json(json!({
                        "conversation_id": conversation.id,
                        "messages": messages,
                        "metadata": conversation.metadata,
                        "created_at": conversation.created_at.elapsed().unwrap_or_default().as_secs(),
                        "updated_at": conversation.updated_at.elapsed().unwrap_or_default().as_secs()
                    })),
                )
            }
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Conversa não encontrada" })),
            ),
        }
    } else {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "Gerenciamento de conversas não está habilitado" })),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::DummyAgent;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    async fn build_test_app() -> Router {
        // Criar um registro com um agente dummy para testes
        let mut registry = AgentRegistry::new();
        registry.register_agent(Box::new(DummyAgent {
            api_key: "test_key".to_string(),
        }));

        // Estado da aplicação para testes
        let app_state = AppState {
            registry: Arc::new(RwLock::new(registry)),
            auth_config: None,
            conversation_manager: None,
        };

        // Configurar roteador
        Router::new()
            .route("/mcp", post(handle_mcp))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_handle_mcp_valid_request() {
        // Construir app de teste
        let app = build_test_app().await;

        // Criar requisição de teste
        let message = MCPMessage::new("dummy:test", json!({"test": "value"}));
        let request = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        // Enviar requisição e verificar resposta
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verificar corpo da resposta
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_message: MCPMessage = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(response_message.command, "dummy_response");
        assert_eq!(response_message.payload, json!({"test": "value"}));
    }

    #[tokio::test]
    async fn test_handle_mcp_invalid_magic() {
        // Construir app de teste
        let app = build_test_app().await;

        // Criar requisição com magic inválido
        let mut message = MCPMessage::new("dummy:test", json!({"test": "value"}));
        message.magic = "INVALID".to_string();

        let request = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        // Enviar requisição e verificar resposta
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verificar corpo da resposta
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_message: MCPMessage = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(response_message.command, "error");
        assert!(response_message.payload["message"]
            .as_str()
            .unwrap()
            .contains("inválido"));
    }

    #[tokio::test]
    async fn test_handle_mcp_agent_not_found() {
        // Construir app de teste
        let app = build_test_app().await;

        // Criar requisição para agente inexistente
        let message = MCPMessage::new("nonexistent:test", json!({"test": "value"}));
        let request = Request::builder()
            .uri("/mcp")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        // Enviar requisição e verificar resposta
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Verificar corpo da resposta
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert!(error_response.error.contains("não foi encontrado"));
    }
}
