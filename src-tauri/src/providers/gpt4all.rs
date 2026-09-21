use serde::Deserialize;

use super::provider::{
    AIProvider,
    ProviderInfo,
    ProviderModel,
};

#[derive(Debug, Deserialize)]
struct GPT4AllModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<GPT4AllModel>,
}

pub struct GPT4AllProvider;

impl GPT4AllProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for GPT4AllProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "gpt4all".to_string(),
            name: "GPT4All".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get(
            "http://127.0.0.1:4891/v1/models"
        )
        .await
        .map_err(|e| {
            format!("Could not connect to GPT4All: {}", e)
        })?;

        Ok(())
    }

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            "http://127.0.0.1:4891/v1/models"
        )
        .await
        .map_err(|e| {
            format!("Could not connect to GPT4All: {}", e)
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "GPT4All returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse GPT4All response: {}",
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
pub async fn get_gpt4all_models()
    -> Result<Vec<ProviderModel>, String>
{
    let provider = GPT4AllProvider::new();

    provider.get_models().await
}