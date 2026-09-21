use serde::Deserialize;

use super::provider::{
    AIProvider,
    ProviderInfo,
    ProviderModel,
};

#[derive(Debug, Deserialize)]
struct LocalAIModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<LocalAIModel>,
}

pub struct LocalAIProvider;

impl LocalAIProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for LocalAIProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "localai".to_string(),
            name: "LocalAI".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get(
            "http://127.0.0.1:8080/v1/models"
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to LocalAI: {}",
                e
            )
        })?;

        Ok(())
    }

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            "http://127.0.0.1:8080/v1/models"
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to LocalAI: {}",
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "LocalAI returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse LocalAI response: {}",
                    e
                )
            })?;

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
pub async fn get_localai_models()
    -> Result<Vec<ProviderModel>, String>
{
    let provider = LocalAIProvider::new();

    provider.get_models().await
}