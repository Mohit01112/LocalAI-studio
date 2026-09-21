use serde::Deserialize;

use super::provider::{
    AIProvider,
    ProviderInfo,
    ProviderModel,
};

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<OpenAIModel>,
}

pub struct VllmProvider;

impl VllmProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for VllmProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "vllm".to_string(),
            name: "vLLM".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get("http://127.0.0.1:8000/v1/models")
            .await
            .map_err(|e| format!("Could not connect to vLLM: {}", e))?;

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get("http://127.0.0.1:8000/v1/models")
            .await
            .map_err(|e| format!("Could not connect to vLLM: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "vLLM returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse vLLM response: {}", e))?;

        Ok(data
            .data
            .into_iter()
            .map(|model| ProviderModel {
                id: model.id.clone(),
                name: model.id,
                size: "Unknown".to_string(),
            })
            .collect())
    }
}

#[tauri::command]
pub async fn get_vllm_models() -> Result<Vec<ProviderModel>, String> {
    let provider = VllmProvider::new();
    provider.get_models().await
}