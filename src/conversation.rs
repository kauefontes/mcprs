//! # Módulo de Gerenciamento de Conversas
//!
//! Este módulo fornece funcionalidades para criar, armazenar e gerenciar
//! históricos de conversas com modelos de IA. Ele permite rastrear mensagens,
//! metadados e limpar automaticamente conversas antigas.
//!
//! ## Exemplo de Uso
//!
//! ```rust
//! use mcprs::conversation::{ConversationManager, Conversation};
//!
//! // Criar um gerenciador de conversas que manterá históricos por 24 horas
//! let manager = ConversationManager::new(24);
//!
//! // Criar uma nova conversa
//! let conversation = manager.create_conversation().unwrap();
//! let conv_id = conversation.id.clone();
//!
//! // Adicionar mensagens à conversa
//! manager.add_message_to_conversation(&conv_id, "user", "Olá, como vai?").unwrap();
//! manager.add_message_to_conversation(&conv_id, "assistant", "Vou bem, obrigado!").unwrap();
//!
//! // Recuperar a conversa posteriormente
//! if let Some(retrieved) = manager.get_conversation(&conv_id) {
//!     println!("Conversa tem {} mensagens", retrieved.messages.len());
//! }
//!
//! // Limpeza periódica (normalmente chamada por um job)
//! let removed = manager.cleanup_old_conversations();
//! println!("{} conversas antigas foram removidas", removed);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Representa uma mensagem individual em uma conversa.
///
/// Cada mensagem tem um papel (role) que identifica se é do usuário,
/// do assistente ou do sistema, além do conteúdo e timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Papel do remetente (user, assistant, system)
    pub role: String,

    /// Conteúdo da mensagem
    pub content: String,

    /// Momento em que a mensagem foi criada
    pub timestamp: SystemTime,
}

/// Representa uma conversa completa entre usuário e assistente.
///
/// Uma conversa contém um ID único, uma sequência de mensagens,
/// metadados opcionais e timestamps de criação e atualização.
#[derive(Clone, Debug)]
pub struct Conversation {
    /// ID único da conversa (UUID)
    pub id: String,

    /// Lista de mensagens na ordem cronológica
    pub messages: Vec<ConversationMessage>,

    /// Metadados opcionais para contexto adicional
    pub metadata: HashMap<String, String>,

    /// Momento de criação da conversa
    pub created_at: SystemTime,

    /// Momento da última atualização da conversa
    pub updated_at: SystemTime,
}

impl Conversation {
    /// Cria uma nova conversa vazia com ID único.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::Conversation;
    ///
    /// let conversation = Conversation::new();
    /// assert!(conversation.messages.is_empty());
    /// ```
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Adiciona uma nova mensagem à conversa.
    ///
    /// # Argumentos
    /// * `role` - Papel do remetente (user, assistant, system)
    /// * `content` - Conteúdo da mensagem
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::Conversation;
    ///
    /// let mut conversation = Conversation::new();
    /// conversation.add_message("user", "Olá!");
    /// conversation.add_message("assistant", "Como posso ajudar?");
    ///
    /// assert_eq!(conversation.messages.len(), 2);
    /// ```
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ConversationMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: SystemTime::now(),
        });
        self.updated_at = SystemTime::now();
    }

    /// Retorna todas as mensagens na conversa.
    ///
    /// # Retorna
    /// Uma fatia contendo as mensagens da conversa
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::Conversation;
    ///
    /// let mut conversation = Conversation::new();
    /// conversation.add_message("user", "Olá!");
    ///
    /// let messages = conversation.get_messages();
    /// assert_eq!(messages.len(), 1);
    /// assert_eq!(messages[0].role, "user");
    /// ```
    pub fn get_messages(&self) -> &[ConversationMessage] {
        &self.messages
    }

    /// Define um valor de metadado para a conversa.
    ///
    /// # Argumentos
    /// * `key` - Chave do metadado
    /// * `value` - Valor do metadado
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::Conversation;
    ///
    /// let mut conversation = Conversation::new();
    /// conversation.set_metadata("language", "pt-br");
    /// conversation.set_metadata("model", "gpt-4");
    ///
    /// assert_eq!(conversation.metadata.get("language").unwrap(), "pt-br");
    /// ```
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

/// Gerenciador de conversas que mantém histórico e limpa conversas antigas.
///
/// O `ConversationManager` é responsável por criar, armazenar, recuperar e
/// limpar conversas, com base em um tempo máximo de retenção configurável.
pub struct ConversationManager {
    /// Mapa de ID para objeto Conversation, compartilhado entre threads
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,

    /// Tempo máximo que uma conversa será mantida após sua última atualização
    max_age: Duration,
}

impl ConversationManager {
    /// Cria um novo gerenciador de conversas com o tempo máximo de retenção especificado.
    ///
    /// # Argumentos
    /// * `max_age_hours` - Tempo máximo de retenção em horas
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    ///
    /// // Reter conversas por 24 horas após a última atualização
    /// let manager = ConversationManager::new(24);
    /// ```
    pub fn new(max_age_hours: u64) -> Self {
        let max_age = Duration::from_secs(max_age_hours * 3600);
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            max_age,
        }
    }

    /// Cria uma nova conversa e a registra no gerenciador.
    ///
    /// # Retorna
    /// * `Ok(Conversation)` - A conversa criada
    /// * `Err(String)` - Mensagem de erro se a operação falhar
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    ///
    /// let manager = ConversationManager::new(24);
    /// let conversation = manager.create_conversation().unwrap();
    /// println!("Nova conversa criada com ID: {}", conversation.id);
    /// ```
    pub fn create_conversation(&self) -> Result<Conversation, String> {
        let conversation = Conversation::new();
        let id = conversation.id.clone();

        if let Ok(mut conversations) = self.conversations.write() {
            conversations.insert(id, conversation.clone());
            Ok(conversation)
        } else {
            Err("Falha ao adquirir lock".to_string())
        }
    }

    /// Recupera uma conversa existente pelo ID.
    ///
    /// # Argumentos
    /// * `id` - O ID da conversa a ser recuperada
    ///
    /// # Retorna
    /// * `Some(Conversation)` - A conversa encontrada
    /// * `None` - Se a conversa não existir
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    ///
    /// let manager = ConversationManager::new(24);
    /// let conversation = manager.create_conversation().unwrap();
    /// let id = conversation.id.clone();
    ///
    /// // Recuperar a conversa
    /// if let Some(retrieved) = manager.get_conversation(&id) {
    ///     println!("Conversa recuperada: {}", retrieved.id);
    /// }
    /// ```
    pub fn get_conversation(&self, id: &str) -> Option<Conversation> {
        if let Ok(conversations) = self.conversations.read() {
            conversations.get(id).cloned()
        } else {
            None
        }
    }

    /// Atualiza uma conversa existente.
    ///
    /// # Argumentos
    /// * `conversation` - A conversa atualizada
    ///
    /// # Retorna
    /// * `Ok(())` - Se a atualização for bem-sucedida
    /// * `Err(String)` - Mensagem de erro se a operação falhar
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    ///
    /// let manager = ConversationManager::new(24);
    /// let mut conversation = manager.create_conversation().unwrap();
    ///
    /// // Modificar a conversa
    /// conversation.add_message("user", "Nova mensagem");
    /// conversation.set_metadata("context", "support");
    ///
    /// // Atualizar no gerenciador
    /// manager.update_conversation(conversation).unwrap();
    /// ```
    pub fn update_conversation(&self, conversation: Conversation) -> Result<(), String> {
        if let Ok(mut conversations) = self.conversations.write() {
            conversations.insert(conversation.id.clone(), conversation);
            Ok(())
        } else {
            Err("Falha ao adquirir lock".to_string())
        }
    }

    /// Adiciona uma mensagem a uma conversa existente.
    ///
    /// # Argumentos
    /// * `conversation_id` - ID da conversa
    /// * `role` - Papel do remetente (user, assistant, system)
    /// * `content` - Conteúdo da mensagem
    ///
    /// # Retorna
    /// * `Ok(())` - Se a adição for bem-sucedida
    /// * `Err(String)` - Mensagem de erro se a operação falhar
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    ///
    /// let manager = ConversationManager::new(24);
    /// let conversation = manager.create_conversation().unwrap();
    /// let id = conversation.id.clone();
    ///
    /// // Adicionar mensagens
    /// manager.add_message_to_conversation(&id, "user", "Olá!").unwrap();
    /// manager.add_message_to_conversation(&id, "assistant", "Como posso ajudar?").unwrap();
    /// ```
    pub fn add_message_to_conversation(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), String> {
        if let Ok(mut conversations) = self.conversations.write() {
            if let Some(conversation) = conversations.get_mut(conversation_id) {
                conversation.add_message(role, content);
                conversation.updated_at = SystemTime::now();
                Ok(())
            } else {
                Err(format!("Conversa {} não encontrada", conversation_id))
            }
        } else {
            Err("Falha ao adquirir lock".to_string())
        }
    }

    /// Remove conversas mais antigas que o tempo máximo de retenção.
    ///
    /// Esta função deve ser chamada periodicamente para limpar conversas antigas.
    ///
    /// # Retorna
    /// O número de conversas removidas
    ///
    /// # Exemplo
    ///
    /// ```
    /// use mcprs::conversation::ConversationManager;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let manager = ConversationManager::new(1); // 1 hora de retenção
    /// let _ = manager.create_conversation().unwrap();
    ///
    /// // Em uma aplicação real, isso seria feito após mais tempo
    /// // e possivelmente em um job programado
    /// thread::sleep(Duration::from_secs(1));
    /// let removed = manager.cleanup_old_conversations();
    /// println!("{} conversas removidas", removed);
    /// ```
    pub fn cleanup_old_conversations(&self) -> usize {
        let now = SystemTime::now();
        let mut count = 0;

        if let Ok(mut conversations) = self.conversations.write() {
            let ids_to_remove: Vec<String> = conversations
                .iter()
                .filter(|(_, conv)| {
                    now.duration_since(conv.updated_at)
                        .map(|duration| duration > self.max_age)
                        .unwrap_or(false)
                })
                .map(|(id, _)| id.clone())
                .collect();

            for id in ids_to_remove {
                conversations.remove(&id);
                count += 1;
            }
        }

        count
    }

    /// Obtém um clone do Arc<RwLock> interno contendo as conversas.
    ///
    /// Útil quando precisa compartilhar o acesso às conversas com outra parte do código.
    ///
    /// # Retorna
    /// Um clone do Arc<RwLock> contendo o mapa de conversas
    pub fn get_arc_clone(&self) -> Arc<RwLock<HashMap<String, Conversation>>> {
        Arc::clone(&self.conversations)
    }
}

impl Clone for ConversationManager {
    fn clone(&self) -> Self {
        Self {
            conversations: Arc::clone(&self.conversations),
            max_age: self.max_age,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_create_and_get_conversation() {
        let manager = ConversationManager::new(24);

        // Criar uma conversa e verificar ID
        let conversation = manager.create_conversation().unwrap();
        let id = conversation.id.clone();

        // Recuperar a conversa pelo ID
        let retrieved = manager.get_conversation(&id).unwrap();
        assert_eq!(retrieved.id, id);
        assert!(retrieved.messages.is_empty());
    }

    #[test]
    fn test_add_message_to_conversation() {
        let manager = ConversationManager::new(24);
        let conversation = manager.create_conversation().unwrap();
        let id = conversation.id.clone();

        // Adicionar mensagens
        manager
            .add_message_to_conversation(&id, "user", "Pergunta 1")
            .unwrap();
        manager
            .add_message_to_conversation(&id, "assistant", "Resposta 1")
            .unwrap();

        // Verificar mensagens
        let retrieved = manager.get_conversation(&id).unwrap();
        assert_eq!(retrieved.messages.len(), 2);
        assert_eq!(retrieved.messages[0].role, "user");
        assert_eq!(retrieved.messages[0].content, "Pergunta 1");
        assert_eq!(retrieved.messages[1].role, "assistant");
        assert_eq!(retrieved.messages[1].content, "Resposta 1");
    }

    #[test]
    fn test_conversation_metadata() {
        let manager = ConversationManager::new(24);
        let mut conversation = manager.create_conversation().unwrap();

        // Adicionar metadados
        conversation.set_metadata("language", "pt-br");
        conversation.set_metadata("model", "gpt-4");

        // Atualizar no gerenciador
        manager.update_conversation(conversation.clone()).unwrap();

        // Verificar metadados
        let retrieved = manager.get_conversation(&conversation.id).unwrap();
        assert_eq!(retrieved.metadata.get("language").unwrap(), "pt-br");
        assert_eq!(retrieved.metadata.get("model").unwrap(), "gpt-4");
    }

    #[test]
    fn test_add_message_nonexistent_conversation() {
        let manager = ConversationManager::new(24);

        // Tentar adicionar mensagem a uma conversa que não existe
        let result = manager.add_message_to_conversation("id-inexistente", "user", "Olá");
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_old_conversations() {
        // Criar gerenciador com tempo muito curto de retenção para testar
        let manager = ConversationManager::new(0); // 0 horas = expiração imediata

        // Criar algumas conversas
        let conv1 = manager.create_conversation().unwrap();
        let _conv2 = manager.create_conversation().unwrap();
        let _conv3 = manager.create_conversation().unwrap();

        // Esperar um pouco para garantir que as conversas expiraram
        thread::sleep(Duration::from_millis(10));

        // Limpar conversas antigas
        let removed = manager.cleanup_old_conversations();

        // Verificar que todas foram removidas
        assert_eq!(removed, 3);
        assert!(manager.get_conversation(&conv1.id).is_none());
    }
}
