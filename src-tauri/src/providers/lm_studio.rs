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

pub struct LMStudioProvider;

impl LMStudioProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for LMStudioProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "lm-studio".to_string(),
            name: "LM Studio".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get(
            "http://127.0.0.1:1234/v1/models",
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to LM Studio: {}",
                e
            )
        })?;

        Ok(())
    }

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            "http://127.0.0.1:1234/v1/models",
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to LM Studio: {}",
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "LM Studio returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse LM Studio response: {}",
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
pub async fn get_lm_studio_models()
    -> Result<Vec<ProviderModel>, String>
{
    let provider = LMStudioProvider::new();

    provider.get_models().await
}