//! Conversation thread and session storage.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::llm::{Message, MessageRole, ToolCall};

/// Default session timeout (30 minutes).
#[allow(dead_code)]
const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// A single conversation session.
///
/// Sessions maintain the full message history and provide methods
/// for adding messages and retrieving state.
#[derive(Debug)]
pub struct Session {
    inner: Arc<SessionInner>,
}

#[derive(Debug)]
struct SessionInner {
    /// Unique session identifier.
    id: String,
    /// Conversation messages.
    messages: RwLock<Vec<Message>>,
    /// Session creation time.
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    /// Last activity time.
    last_activity: RwLock<DateTime<Utc>>,
    /// Optional system prompt.
    system_prompt: RwLock<Option<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: String,    // RFC3339
    pub last_activity: String, // RFC3339
    pub system_prompt: Option<String>,
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state = self.to_state();
        state.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Session {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state = SessionState::deserialize(deserializer)?;
        Ok(Session::from_state(state))
    }
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Session {
    /// Create a new session with the given ID.
    fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            inner: Arc::new(SessionInner {
                id,
                messages: RwLock::new(Vec::new()),
                created_at: now,
                last_activity: RwLock::new(now),
                system_prompt: RwLock::new(None),
            }),
        }
    }

    pub fn to_state(&self) -> SessionState {
        SessionState {
            id: self.inner.id.clone(),
            messages: self.inner.messages.read().unwrap().clone(),
            created_at: self.inner.created_at.to_rfc3339(),
            last_activity: self.inner.last_activity.read().unwrap().to_rfc3339(),
            system_prompt: self.inner.system_prompt.read().unwrap().clone(),
        }
    }

    pub fn from_state(state: SessionState) -> Self {
        let created_at = DateTime::parse_from_rfc3339(&state.created_at)
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
            .with_timezone(&Utc);
        let last_activity = DateTime::parse_from_rfc3339(&state.last_activity)
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
            .with_timezone(&Utc);

        Self {
            inner: Arc::new(SessionInner {
                id: state.id,
                messages: RwLock::new(state.messages),
                created_at,
                last_activity: RwLock::new(last_activity),
                system_prompt: RwLock::new(state.system_prompt),
            }),
        }
    }

    /// Get the session ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    /// Set the system prompt for this session.
    #[allow(dead_code)]
    pub fn set_system_prompt(&self, prompt: impl Into<String>) {
        let mut guard = self.inner.system_prompt.write().unwrap();
        *guard = Some(prompt.into());
        self.touch();
    }

    /// Get the system prompt if set.
    #[must_use]
    pub fn system_prompt(&self) -> Option<String> {
        self.inner.system_prompt.read().unwrap().clone()
    }

    /// Add a user message to the conversation.
    pub fn add_user_message(&self, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::User,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add an assistant message to the conversation.
    #[allow(dead_code)]
    pub fn add_assistant_message(&self, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add an assistant message with tool calls.
    #[allow(dead_code)]
    pub fn add_assistant_with_tool_calls(
        &self,
        content: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) {
        let msg = Message {
            role: MessageRole::Assistant,
            content: content.unwrap_or_default(),
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        };
        self.add_message(msg);
    }

    /// Add a tool result message.
    #[allow(dead_code)]
    pub fn add_tool_result(&self, tool_call_id: impl Into<String>, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add a message to the conversation.
    pub fn add_message(&self, message: Message) {
        let mut guard = self.inner.messages.write().unwrap();
        guard.push(message);
        drop(guard);
        self.touch();
    }

    /// Get all messages in the conversation.
    #[must_use]
    pub fn messages(&self) -> Vec<Message> {
        self.inner.messages.read().unwrap().clone()
    }

    /// Get all messages including the system prompt.
    #[must_use]
    pub fn messages_with_system(&self) -> Vec<Message> {
        let mut result = Vec::new();

        if let Some(prompt) = self.system_prompt() {
            result.push(Message {
                role: MessageRole::System,
                content: prompt,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        result.extend(self.messages());
        result
    }

    /// Get the number of messages in the conversation.
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.inner.messages.read().unwrap().len()
    }

    /// Clear all messages from the session.
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut guard = self.inner.messages.write().unwrap();
        guard.clear();
        self.touch();
    }

    /// Update the last activity timestamp.
    fn touch(&self) {
        let mut guard = self.inner.last_activity.write().unwrap();
        *guard = Utc::now();
    }

    /// Check if the session has expired.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        self.is_expired_with_timeout(DEFAULT_SESSION_TIMEOUT)
    }

    /// Check if the session has expired with a custom timeout.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_expired_with_timeout(&self, timeout: Duration) -> bool {
        let last = *self.inner.last_activity.read().unwrap();
        // Convert Duration to chrono::Duration?
        // chrono subtraction: now - last -> Duration
        // We want (now - last) > timeout
        let now = Utc::now();
        if let Ok(duration) = (now - last).to_std() {
            duration > timeout
        } else {
            // Negative duration means clock skew or "last" is in future.
            false
        }
    }

    /// Get the session age.
    #[must_use]
    #[allow(dead_code)]
    pub fn age(&self) -> Duration {
        let now = Utc::now();
        (now - self.inner.created_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Thread-safe store for sessions.
///
/// Provides methods for creating, retrieving, and cleaning up sessions.
#[derive(Debug, Clone)]
pub struct SessionStore {
    inner: Arc<SessionStoreInner>,
}

#[derive(Debug)]
struct SessionStoreInner {
    sessions: RwLock<HashMap<String, Session>>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    /// Create a new session store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SessionStoreInner {
                sessions: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Create a new session and return it.
    #[must_use]
    pub fn create(&self) -> Session {
        let id = Uuid::new_v4().to_string();
        self.create_with_id(id)
    }

    /// Create a new session with a specific ID.
    #[must_use]
    pub fn create_with_id(&self, id: impl Into<String>) -> Session {
        let id = id.into();
        let session = Session::new(id.clone());
        let mut guard = self.inner.sessions.write().unwrap();
        guard.insert(id, session.clone());
        session
    }

    /// Get a session by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Session> {
        let guard = self.inner.sessions.read().unwrap();
        guard.get(id).cloned()
    }

    /// Get a session by ID, creating it if it doesn't exist.
    #[must_use]
    pub fn get_or_create(&self, id: &str) -> Session {
        // Try read-only first
        {
            let guard = self.inner.sessions.read().unwrap();
            if let Some(session) = guard.get(id) {
                return session.clone();
            }
        }

        // Create if not exists
        self.create_with_id(id)
    }

    /// Remove a session by ID.
    pub fn remove(&self, id: &str) -> Option<Session> {
        let mut guard = self.inner.sessions.write().unwrap();
        guard.remove(id)
    }

    /// Get the number of active sessions.
    #[must_use]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.sessions.read().unwrap().len()
    }

    /// Check if there are no sessions.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all expired sessions.
    ///
    /// Returns the number of sessions removed.
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) -> usize {
        self.cleanup_expired_with_timeout(DEFAULT_SESSION_TIMEOUT)
    }

    /// Remove sessions that have been inactive longer than the timeout.
    #[allow(dead_code)]
    pub fn cleanup_expired_with_timeout(&self, timeout: Duration) -> usize {
        let mut guard = self.inner.sessions.write().unwrap();
        let before = guard.len();
        guard.retain(|_, session| !session.is_expired_with_timeout(timeout));
        before - guard.len()
    }

    /// List all session IDs.
    #[must_use]
    pub fn list_ids(&self) -> Vec<String> {
        self.inner
            .sessions
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let session = Session::new("test-123".to_string());

        assert_eq!(session.id(), "test-123");
        assert_eq!(session.message_count(), 0);

        session.add_user_message("Hello");
        assert_eq!(session.message_count(), 1);

        session.add_assistant_message("Hi there!");
        assert_eq!(session.message_count(), 2);

        let messages = session.messages();
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_session_store() {
        let store = SessionStore::new();

        assert!(store.is_empty());

        let session = store.create();
        assert_eq!(store.len(), 1);

        let retrieved = store.get(session.id()).unwrap();
        assert_eq!(retrieved.id(), session.id());

        store.remove(session.id());
        assert!(store.is_empty());
    }

    #[test]
    fn test_system_prompt() {
        let session = Session::new("test".to_string());

        assert!(session.system_prompt().is_none());

        session.set_system_prompt("You are a helpful assistant.");
        assert_eq!(
            session.system_prompt().unwrap(),
            "You are a helpful assistant."
        );

        let messages = session.messages_with_system();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::System);
    }
}
