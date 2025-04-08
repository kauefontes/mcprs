//! # Módulo de Streaming
//!
//! Este módulo fornece suporte para lidar com respostas em streaming dos modelos de IA.
//! Ele permite processar streams de bytes (como SSE de APIs) em tokens estruturados
//! que podem ser consumidos incrementalmente.
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use futures::StreamExt;
//! use mcprs::streaming::{process_json_stream, StreamingToken};
//! use reqwest::Client;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Iniciar uma requisição em streaming
//! let client = Client::new();
//! let stream = client
//!     .post("https://api.example.com/streaming")
//!     .send()
//!     .await?
//!     .bytes_stream();
//!
//! // Processar o stream como tokens
//! let mut token_stream = process_json_stream::<_, serde_json::Value>(stream).await?;
//!
//! // Consumir os tokens
//! while let Some(token_result) = token_stream.next().await {
//!     match token_result {
//!         Ok(token) => {
//!             print!("{}", token.content);
//!             if token.is_finish {
//!                 break;
//!             }
//!         }
//!         Err(e) => eprintln!("Erro: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::agent::MCPError;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Representa um token ou fragmento de uma resposta em streaming.
///
/// Cada token contém uma parte do conteúdo da resposta, uma flag indicando
/// se é o último token, e metadados opcionais.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingToken {
    /// Conteúdo do token (parte da resposta)
    pub content: String,

    /// Indica se este é o último token do stream
    pub is_finish: bool,

    /// Metadados opcionais associados ao token
    pub metadata: Option<Value>,
}

/// Tipo para representar um stream assíncrono de tokens de resposta.
///
/// Este tipo é um stream de resultados contendo tokens ou erros.
pub type TokenStream = Pin<Box<dyn Stream<Item = Result<StreamingToken, MCPError>> + Send>>;

/// Cria um novo stream de tokens a partir de um receiver de canal Tokio.
///
/// # Argumentos
/// * `receiver` - Um receptor de canal de tokens
///
/// # Retorna
/// Um stream tipado que pode ser iterado de forma assíncrona
///
/// # Exemplo
///
/// ```rust,no_run
/// use tokio::sync::mpsc;
/// use mcprs::agent::MCPError;
/// use mcprs::streaming::{StreamingToken, create_token_stream};
///
/// # async fn example() {
/// let (tx, rx) = mpsc::channel(100);
///
/// // Em algum lugar do código, enviamos tokens para o canal
/// tx.send(Ok(StreamingToken {
///     content: "Parte da resposta".to_string(),
///     is_finish: false,
///     metadata: None,
/// })).await.unwrap();
///
/// // Criar um stream a partir do receptor
/// let token_stream = create_token_stream(rx);
/// # }
/// ```
pub fn create_token_stream(
    receiver: mpsc::Receiver<Result<StreamingToken, MCPError>>,
) -> TokenStream {
    let stream = ReceiverStream::new(receiver);
    Box::pin(stream)
}

/// Processa um stream de bytes e converte para tokens JSON estruturados.
///
/// Esta função é útil para processar respostas em streaming de APIs como OpenAI
/// que enviam eventos SSE ou chunks JSON por linha.
///
/// # Argumentos
/// * `stream` - Um stream de bytes (geralmente de uma resposta HTTP)
///
/// # Tipo Genérico
/// * `T` - O tipo a ser desserializado de cada chunk JSON
///
/// # Retorna
/// * `Ok(TokenStream)` - Stream de tokens processados
/// * `Err(MCPError)` - Se ocorrer um erro ao configurar o processamento
///
/// # Exemplo
///
/// ```rust,no_run
/// use futures::StreamExt;
/// use mcprs::streaming::process_json_stream;
/// use reqwest::Client;
/// use serde_json::Value;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Iniciar uma requisição em streaming
/// let client = Client::new();
/// let response = client.get("https://api.example.com/stream").send().await?;
/// let byte_stream = response.bytes_stream();
///
/// // Processar como stream de valores JSON
/// let mut token_stream = process_json_stream::<_, Value>(byte_stream).await?;
///
/// // Consumir tokens
/// while let Some(Ok(token)) = token_stream.next().await {
///     print!("{}", token.content);
///     if token.is_finish {
///         break;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn process_json_stream<S, T>(stream: S) -> Result<TokenStream, MCPError>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    T: for<'de> Deserialize<'de> + Send + 'static + Debug,
{
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut stream = Box::pin(stream);
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    buffer.push_str(&chunk_str);

                    // Processar buffer para extrair objetos JSON completos
                    while let Some(pos) = buffer.find('\n') {
                        // Converter para String para ter propriedade dos dados
                        let line = buffer[..pos].trim().to_string();

                        // Agora line é proprietária dos dados, podemos modificar buffer seguramente
                        buffer = buffer[pos + 1..].to_string();

                        if line.is_empty() || line == "data: [DONE]" {
                            continue;
                        }

                        // Remover prefixos comuns como "data: "
                        let json_str = line.strip_prefix("data: ").unwrap_or(&line);

                        match serde_json::from_str::<T>(json_str) {
                            Ok(parsed) => {
                                let token = StreamingToken {
                                    content: format!("{:?}", parsed),
                                    is_finish: false,
                                    metadata: None,
                                };

                                if tx.send(Ok(token)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(Err(MCPError::InternalAgentError(format!(
                                        "Erro ao desserializar: {}",
                                        e
                                    ))))
                                    .await;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(MCPError::InternalAgentError(format!(
                            "Erro de rede: {}",
                            e
                        ))))
                        .await;
                    break;
                }
            }
        }

        // Sinalizar que o stream terminou
        let _ = tx
            .send(Ok(StreamingToken {
                content: String::new(),
                is_finish: true,
                metadata: None,
            }))
            .await;
    });

    Ok(create_token_stream(rx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn test_create_token_stream() {
        let (tx, rx) = mpsc::channel(10);

        // Enviar alguns tokens
        tx.send(Ok(StreamingToken {
            content: "Token 1".to_string(),
            is_finish: false,
            metadata: None,
        }))
        .await
        .unwrap();

        tx.send(Ok(StreamingToken {
            content: "Token 2".to_string(),
            is_finish: false,
            metadata: None,
        }))
        .await
        .unwrap();

        tx.send(Ok(StreamingToken {
            content: "".to_string(),
            is_finish: true,
            metadata: None,
        }))
        .await
        .unwrap();

        // Criar stream e coletar tokens
        let mut token_stream = create_token_stream(rx);
        let mut collected_content = Vec::new();
        let mut saw_finish = false;

        while let Some(token_result) = token_stream.next().await {
            let token = token_result.unwrap();
            if token.is_finish {
                saw_finish = true;
                break;
            }
            collected_content.push(token.content);
        }

        assert_eq!(collected_content, vec!["Token 1", "Token 2"]);
        assert!(saw_finish);
    }

    #[derive(Deserialize, Debug)]
    struct TestResponse {
        #[allow(dead_code)]
        text: String,
    }

    #[tokio::test]
    async fn test_process_json_stream() {
        // Criar um stream simulado com chunks JSON
        let chunks = vec![
            Ok(bytes::Bytes::from(r#"{"text":"Parte 1"}"#)),
            Ok(bytes::Bytes::from("\n")),
            Ok(bytes::Bytes::from(r#"{"text":"Parte 2"}"#)),
            Ok(bytes::Bytes::from("\n")),
            Ok(bytes::Bytes::from("data: [DONE]\n")),
        ];

        let mock_stream = stream::iter(chunks);

        // Processar o stream
        let mut token_stream = process_json_stream::<_, TestResponse>(mock_stream)
            .await
            .unwrap();

        // Coletar tokens
        let mut tokens = Vec::new();
        while let Some(token_result) = token_stream.next().await {
            let token = token_result.unwrap();
            if token.is_finish {
                break;
            }
            tokens.push(token.content);
        }

        // Verificar tokens coletados
        assert_eq!(tokens.len(), 2);
        assert!(tokens[0].contains("Parte 1"));
        assert!(tokens[1].contains("Parte 2"));
    }
}
