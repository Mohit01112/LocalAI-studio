use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderModel {
    pub id: String,
    pub name: String,
    pub size: String,
}

/// A single message in a conversation.
///
/// This is provider-neutral. Each runtime will translate
/// these messages into its own API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Generic chat request used by LocalAI Studio.
///
/// The frontend sends this structure regardless of
/// which runtime is being used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,

    #[serde(default)]
    pub temperature: Option<f32>,

    #[serde(default)]
    pub stream: bool,
}

/// Generic response returned by every provider.
///
/// Provider-specific response formats are converted
/// into this structure before reaching the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,

    #[serde(default)]
    pub done: bool,

    #[serde(default)]
    pub provider: Option<String>,

    #[serde(default)]
    pub model: Option<String>,
}

#[async_trait::async_trait]
pub trait AIProvider {
    async fn info(&self) -> ProviderInfo;

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String>;

    async fn connect(&self) -> Result<(), String> {
        Ok(())
    }

    async fn start(&self) -> Result<(), String> {
        Err(
            "Start is not supported by this provider."
                .to_string(),
        )
    }

    async fn stop(&self) -> Result<(), String> {
        Err(
            "Stop is not supported by this provider."
                .to_string(),
        )
    }

    async fn download_model(
        &self,
        _model: &str,
    ) -> Result<(), String> {
        Err(
            "Model downloads are not supported by this provider yet."
                .to_string(),
        )
    }

    async fn delete_model(
        &self,
        _model: &str,
    ) -> Result<(), String> {
        Err(
            "Model deletion is not supported by this provider yet."
                .to_string(),
        )
    }

    /// Provider-neutral chat interface.
    ///
    /// LocalAI Studio sends the same ChatRequest to every
    /// provider. Each provider implementation is responsible
    /// for translating it into its own native API format.
    async fn chat(
        &self,
        _request: &ChatRequest,
    ) -> Result<ChatResponse, String> {
        Err(
            "Chat is not supported by this provider yet."
                .to_string(),
        )
    }
}