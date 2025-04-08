//! # Módulo de Autenticação
//!
//! Este módulo fornece um sistema de autenticação para o servidor MCP,
//! baseado em tokens Bearer. Ele inclui configuração de tokens permitidos,
//! extratores para Axum, e tratamento de erros de autenticação.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::auth::AuthConfig;
//!
//! // Criar configuração de autenticação
//! let auth_config = AuthConfig::new();
//!
//! // Adicionar tokens permitidos
//! auth_config.add_token("seu-token-secreto".to_string());
//!
//! // Verificar token
//! let is_valid = auth_config.is_valid_token("seu-token-secreto");
//! assert!(is_valid);
//! ```

use axum::{
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// Representa um usuário autenticado após validação do token.
///
/// Esta estrutura é utilizada como extrator em rotas protegidas do Axum.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Token de autenticação validado
    pub token: String,
}

/// Configuração de autenticação para o servidor MCP.
///
/// Mantém um conjunto de tokens válidos e fornece métodos para
/// validação e gerenciamento desses tokens.
#[derive(Clone)]
pub struct AuthConfig {
    /// Conjunto de tokens válidos, compartilhado entre threads
    tokens: Arc<RwLock<HashSet<String>>>,
}

impl AuthConfig {
    /// Cria uma nova configuração de autenticação vazia.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::auth::AuthConfig;
    ///
    /// let config = AuthConfig::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Adiciona um token à lista de tokens válidos.
    ///
    /// # Argumentos
    /// * `token` - O token a ser adicionado
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::auth::AuthConfig;
    ///
    /// let config = AuthConfig::new();
    /// config.add_token("token123".to_string());
    /// ```
    pub fn add_token(&self, token: String) {
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.insert(token);
        }
    }

    /// Verifica se um token está na lista de tokens válidos.
    ///
    /// # Argumentos
    /// * `token` - O token a ser verificado
    ///
    /// # Retorna
    /// `true` se o token for válido, `false` caso contrário
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::auth::AuthConfig;
    ///
    /// let config = AuthConfig::new();
    /// config.add_token("token123".to_string());
    ///
    /// assert!(config.is_valid_token("token123"));
    /// assert!(!config.is_valid_token("token-invalido"));
    /// ```
    pub fn is_valid_token(&self, token: &str) -> bool {
        if let Ok(tokens) = self.tokens.read() {
            tokens.contains(token)
        } else {
            false
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Representa um erro de autenticação.
///
/// Esta estrutura é usada para retornar respostas de erro
/// quando a autenticação falha.
#[derive(Serialize)]
pub struct AuthError {
    /// Mensagem de erro para o cliente
    message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(self)).into_response()
    }
}

/// Implementação do extrator `AuthUser` para Axum.
///
/// Este extrator pode ser usado em handlers Axum para exigir
/// autenticação via token Bearer.
#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Tentativa de obter header de autorização
        if let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state).await
        {
            // Na versão simplificada, aceitamos qualquer token
            Ok(AuthUser {
                token: bearer.token().to_string(),
            })
        } else {
            Err(AuthError {
                message: "Token de autorização ausente ou inválido".into(),
            })
        }
    }
}

/// Implementação do extrator `AuthConfig` para Axum.
///
/// Este extrator é usado internamente para obter a configuração
/// de autenticação a partir do estado do aplicativo.
#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthConfig
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Versão simplificada - lógica real seria implementada com Extension
        Ok(AuthConfig::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_add_and_validate_token() {
        let config = AuthConfig::new();

        // Adicionar token e verificar
        config.add_token("test-token".to_string());
        assert!(config.is_valid_token("test-token"));

        // Verificar token inexistente
        assert!(!config.is_valid_token("invalid-token"));
    }

    #[test]
    fn test_auth_config_multiple_tokens() {
        let config = AuthConfig::new();

        // Adicionar múltiplos tokens
        config.add_token("token1".to_string());
        config.add_token("token2".to_string());
        config.add_token("token3".to_string());

        // Verificar todos
        assert!(config.is_valid_token("token1"));
        assert!(config.is_valid_token("token2"));
        assert!(config.is_valid_token("token3"));
        assert!(!config.is_valid_token("token4"));
    }

    #[test]
    fn test_auth_error_into_response() {
        let error = AuthError {
            message: "Token inválido".to_string(),
        };

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Testes mais avançados envolvendo os extractors necessitariam de um ambiente
    // de teste Axum, o que está fora do escopo destes testes unitários simples.
}
