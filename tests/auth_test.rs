use mcprs::auth::AuthConfig;

#[test]
fn test_auth_config_token_management() {
    let config = AuthConfig::new();

    // Inicialmente não deve ter tokens
    assert!(!config.is_valid_token("any-token"));

    // Adicionar um token e verificar
    config.add_token("token1".to_string());
    assert!(config.is_valid_token("token1"));
    assert!(!config.is_valid_token("token2"));

    // Adicionar mais tokens
    config.add_token("token2".to_string());
    config.add_token("token3".to_string());

    // Verificar todos
    assert!(config.is_valid_token("token1"));
    assert!(config.is_valid_token("token2"));
    assert!(config.is_valid_token("token3"));
    assert!(!config.is_valid_token("token4"));
}

#[test]
fn test_auth_config_clone() {
    let config = AuthConfig::new();
    config.add_token("secret-token".to_string());

    // Clonar a configuração
    let cloned_config = config.clone();

    // Verificar se o token existe na configuração clonada
    assert!(cloned_config.is_valid_token("secret-token"));

    // Adicionar token na original e verificar se está na clonada
    config.add_token("another-token".to_string());
    assert!(
        cloned_config.is_valid_token("another-token"),
        "O token deveria existir na config clonada pois Arc é usado internamente"
    );
}

// Testes avançados envolvendo AuthUser e FromRequestParts
// necessitariam de um ambiente de teste Axum completo
// e seriam mais complexos, por isso foram omitidos aqui.
