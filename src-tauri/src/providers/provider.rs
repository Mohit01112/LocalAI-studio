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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAttachment {
    pub name: String,
    pub mime_type: String,
    pub data: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,

    pub messages: Vec<ChatMessage>,

    #[serde(default)]
    pub temperature: Option<f32>,

    #[serde(default)]
    pub stream: bool,

    #[serde(default)]
    pub attachments: Vec<ChatAttachment>,
}

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