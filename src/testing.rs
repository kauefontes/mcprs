use async_trait::async_trait;
use mockall::automock;
use reqwest::Response;

/// Define uma interface abstrata para HTTP clients
#[automock]
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn post(
        &self,
        url: String,
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error>;

    async fn get(
        &self,
        url: String,
        headers: Vec<(String, String)>,
    ) -> Result<Response, reqwest::Error>;
}

/// Implementação real do HttpClient usando reqwest
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

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

/// Factory trait para criar instâncias de HttpClient
#[automock]
pub trait HttpClientFactory {
    fn create_client(&self) -> Box<dyn HttpClient>;
}

/// Implementação real da factory
pub struct ReqwestClientFactory;

impl HttpClientFactory for ReqwestClientFactory {
    fn create_client(&self) -> Box<dyn HttpClient> {
        Box::new(ReqwestClient::new())
    }
}
