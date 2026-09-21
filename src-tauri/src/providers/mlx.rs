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

pub struct MlxProvider;

impl MlxProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for MlxProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "mlx".to_string(),
            name: "MLX / MLX-LM".to_string(),
            installed: cfg!(target_os = "macos"),
            running: false,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get("http://127.0.0.1:8080/v1/models")
            .await
            .map_err(|e| format!("Could not connect to MLX server: {}", e))?;

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get("http://127.0.0.1:8080/v1/models")
            .await
            .map_err(|e| format!("Could not connect to MLX server: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "MLX server returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MLX response: {}", e))?;

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
pub async fn get_mlx_models() -> Result<Vec<ProviderModel>, String> {
    let provider = MlxProvider::new();
    provider.get_models().await
}