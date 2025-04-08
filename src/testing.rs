//! # Módulo de Utilitários de Teste
//!
//! Este módulo fornece ferramentas para auxiliar no teste de componentes
//! que dependem de HTTP, permitindo o mock de chamadas HTTP para isolamento
//! de testes.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use mcprs::testing::{HttpClient, MockHttpClient};
//! use mockall::predicate;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar um cliente HTTP mockado
//! let mut mock_client = MockHttpClient::new();
//!
//! // Configurar expectativas do mock
//! mock_client
//!     .expect_post()
//!     .with(
//!         predicate::eq("https://api.example.com/endpoint".to_string()),
//!         predicate::always(),
//!         predicate::always(),
//!     )
//!     .times(1)
//!     .returning(|_, _, _| {
//!         // Retornar uma resposta simulada
//!         // ...
//!         # Ok(reqwest::Response::from(
//!         #    http::Response::builder()
//!         #        .status(200)
//!         #        .body("{}".to_string())
//!         #        .unwrap(),
//!         # ))
//!     });
//!
//! // Usar o mock em um componente que depende de HTTP
//! // ...
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use mockall::automock;
use reqwest::Response;

/// Define uma interface abstrata para clientes HTTP.
///
/// Esta trait permite abstrair operações HTTP comuns, tornando
/// mais fácil mock e testar componentes que fazem requisições HTTP.
#[automock]
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Executa uma requisição HTTP POST.
    ///
    /// # Argumentos
    /// * `url` - URL para a requisição
    /// * `body` - Corpo da requisição como bytes
    /// * `headers` - Cabeçalhos HTTP como pares (nome, valor)
    ///
    /// # Retorna
    /// * `Ok(Response)` - A resposta HTTP
    /// * `Err(reqwest::Error)` - Se ocorrer um erro na requisição
    async fn post(
        &self,
        url: String,
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error>;

    /// Executa uma requisição HTTP GET.
    ///
    /// # Argumentos
    /// * `url` - URL para a requisição
    /// * `headers` - Cabeçalhos HTTP como pares (nome, valor)
    ///
    /// # Retorna
    /// * `Ok(Response)` - A resposta HTTP
    /// * `Err(reqwest::Error)` - Se ocorrer um erro na requisição
    async fn get(
        &self,
        url: String,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error>;
}

/// Implementação concreta de `HttpClient` usando o crate reqwest.
///
/// Esta é a implementação padrão utilizada em produção.
pub struct ReqwestClient {
    /// Cliente reqwest subjacente
    client: reqwest::Client,
}

impl ReqwestClient {
    /// Cria uma nova instância de `ReqwestClient` com configuração padrão.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::testing::ReqwestClient;
    ///
    /// let client = ReqwestClient::new();
    /// ```
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Cria uma nova instância com um cliente reqwest específico.
    ///
    /// # Argumentos
    /// * `client` - Um cliente reqwest pré-configurado
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::testing::ReqwestClient;
    ///
    /// let reqwest_client = reqwest::Client::builder()
    ///     .timeout(std::time::Duration::from_secs(30))
    ///     .build()
    ///     .unwrap();
    ///
    /// let client = ReqwestClient::with_client(reqwest_client);
    /// ```
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn post(
        &self,
        url: String,
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error> {
        let mut request = self.client.post(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        request.body(body).send().await
    }

    async fn get(
        &self,
        url: String,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error> {
        let mut request = self.client.get(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        request.send().await
    }
}

/// Factory trait para criar instâncias de HttpClient.
///
/// Esta trait permite a injeção de dependência de fábricas
/// de HttpClient, facilitando testes.
#[automock]
pub trait HttpClientFactory {
    /// Cria uma nova instância de HttpClient.
    ///
    /// # Retorna
    /// Uma implementação concreta de HttpClient encapsulada em Box
    fn create_client(&self) -> Box<dyn HttpClient>;
}

/// Implementação padrão de HttpClientFactory que cria ReqwestClient.
pub struct ReqwestClientFactory;

impl HttpClientFactory for ReqwestClientFactory {
    fn create_client(&self) -> Box<dyn HttpClient> {
        Box::new(ReqwestClient::new())
    }
}

impl Default for ReqwestClientFactory {
    fn default() -> Self {
        Self
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate;

    // Teste apenas para demonstrar como usar o mock
    #[tokio::test]
    async fn test_mock_http_client() {
        let mut mock = MockHttpClient::new();

        // Configurar o comportamento do mock
        mock.expect_post()
            .with(
                predicate::eq("https://test.example.com".to_string()),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| {
                Ok(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body("Test Response")
                        .unwrap(),
                ))
            });

        // Usar o mock
        let result = mock
            .post(
                "https://test.example.com".to_string(),
                b"test body".to_vec(),
                vec![("Content-Type".to_string(), "text/plain".to_string())],
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_mock_http_client_factory() {
        let mut mock_factory = MockHttpClientFactory::new();

        // Configurar a fábrica para retornar um mock configurado
        mock_factory.expect_create_client().times(1).returning(|| {
            // Criamos e configuramos um novo mock dentro do closure
            let mut new_mock = MockHttpClient::new();
            new_mock.expect_get().returning(|_, _| {
                Ok(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body("Factory Test")
                        .unwrap(),
                ))
            });
            Box::new(new_mock)
        });

        // Criar cliente via fábrica
        let client = mock_factory.create_client();

        // O teste real seria mais elaborado, isso é apenas para demonstrar o uso
        assert!(client
            .get("https://example.com".to_string(), vec![])
            .await
            .is_ok());
    }
}
