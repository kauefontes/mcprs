use futures::stream;
use futures::StreamExt;
use mcprs::agent::MCPError;
use mcprs::streaming::{create_token_stream, process_json_stream, StreamingToken};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_token_stream_basic() {
    let (tx, rx) = mpsc::channel(10);

    // Enviar tokens para o canal
    tx.send(Ok(StreamingToken {
        content: "Primeiro token".to_string(),
        is_finish: false,
        metadata: None,
    }))
    .await
    .unwrap();

    tx.send(Ok(StreamingToken {
        content: "Segundo token".to_string(),
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

    // Criar e testar o stream
    let mut stream = create_token_stream(rx);

    // Primeiro token
    let token1 = stream.next().await.unwrap().unwrap();
    assert_eq!(token1.content, "Primeiro token");
    assert!(!token1.is_finish);

    // Segundo token
    let token2 = stream.next().await.unwrap().unwrap();
    assert_eq!(token2.content, "Segundo token");
    assert!(!token2.is_finish);

    // Token final
    let token3 = stream.next().await.unwrap().unwrap();
    assert!(token3.is_finish);
}

#[tokio::test]
async fn test_token_stream_with_error() {
    let (tx, rx) = mpsc::channel(10);

    // Enviar um token normal
    tx.send(Ok(StreamingToken {
        content: "Token normal".to_string(),
        is_finish: false,
        metadata: None,
    }))
    .await
    .unwrap();

    // Enviar um erro
    tx.send(Err(MCPError::InternalAgentError(
        "Erro de teste".to_string(),
    )))
    .await
    .unwrap();

    // Enviar token final
    tx.send(Ok(StreamingToken {
        content: "".to_string(),
        is_finish: true,
        metadata: None,
    }))
    .await
    .unwrap();

    // Testar o stream
    let mut stream = create_token_stream(rx);

    // Primeiro token (ok)
    let result1 = stream.next().await.unwrap();
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().content, "Token normal");

    // Segundo resultado (erro)
    let result2 = stream.next().await.unwrap();
    assert!(result2.is_err());
    assert!(matches!(result2, Err(MCPError::InternalAgentError(e)) if e == "Erro de teste"));

    // Terceiro token (token final)
    let result3 = stream.next().await.unwrap();
    assert!(result3.is_ok());
    assert!(result3.unwrap().is_finish);
}

#[derive(Debug, Serialize, Deserialize)]
struct TestJsonChunk {
    text: String,
    index: u32,
}

#[tokio::test]
async fn test_process_json_stream() {
    // Criar um stream simulado com chunks JSON
    let chunks = vec![
        Ok(bytes::Bytes::from(r#"{"text":"Chunk 1","index":1}"#)),
        Ok(bytes::Bytes::from("\n")),
        Ok(bytes::Bytes::from(r#"{"text":"Chunk 2","index":2}"#)),
        Ok(bytes::Bytes::from("\n")),
        Ok(bytes::Bytes::from(r#"data: {"text":"Chunk 3","index":3}"#)),
        Ok(bytes::Bytes::from("\n")),
        Ok(bytes::Bytes::from("data: [DONE]\n")),
    ];

    let mock_stream = stream::iter(chunks);

    // Processar o stream
    let mut token_stream = process_json_stream::<_, TestJsonChunk>(mock_stream)
        .await
        .unwrap();

    // Verificar tokens
    let mut tokens = Vec::new();

    while let Some(token_result) = token_stream.next().await {
        match token_result {
            Ok(token) => {
                if token.is_finish {
                    break;
                }
                tokens.push(token.content);
            }
            Err(e) => panic!("Não deveria ter erro: {}", e),
        }
    }

    // Deveria ter 3 tokens
    assert_eq!(tokens.len(), 3);
    assert!(tokens[0].contains("Chunk 1"));
    assert!(tokens[0].contains("index: 1"));
    assert!(tokens[1].contains("Chunk 2"));
    assert!(tokens[2].contains("Chunk 3"));
}

#[tokio::test]
async fn test_process_json_stream_with_errors() {
    // Criar um stream com erro
    let chunks = vec![
        Ok(bytes::Bytes::from(r#"{"text":"Chunk válido","index":1}"#)),
        Ok(bytes::Bytes::from("\n")),
        Ok(bytes::Bytes::from(
            r#"{"invalid_json: esta linha não é um JSON válido"#,
        )),
        Ok(bytes::Bytes::from("\n")),
        // Usar uma forma alternativa de criar um erro, já que o reqwest::Error é complexo
        // Neste caso, vamos apenas testar a parte de parsing JSON inválido e ignorar o erro de rede
    ];

    let mock_stream = stream::iter(chunks);

    // Processar o stream
    let mut token_stream = process_json_stream::<_, TestJsonChunk>(mock_stream)
        .await
        .unwrap();

    // Primeiro token (válido)
    let token1 = token_stream.next().await.unwrap();
    assert!(token1.is_ok());
    assert!(token1.unwrap().content.contains("Chunk válido"));

    // Segundo resultado (erro de parsing)
    let token2 = token_stream.next().await.unwrap();
    assert!(token2.is_err());
    assert!(matches!(token2, Err(MCPError::InternalAgentError(e)) if e.contains("desserializar")));

    // Token final, gerado após o processamento
    let token3 = token_stream.next().await.unwrap();
    assert!(token3.is_ok());
    assert!(token3.unwrap().is_finish);
}
