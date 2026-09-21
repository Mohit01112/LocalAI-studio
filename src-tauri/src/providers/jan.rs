use serde::Deserialize;

use super::provider::{
    AIProvider,
    ProviderInfo,
    ProviderModel,
};

#[derive(Debug, Deserialize)]
struct JanModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<JanModel>,
}

pub struct JanProvider;

impl JanProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for JanProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "jan".to_string(),
            name: "Jan".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get(
            "http://127.0.0.1:1337/v1/models"
        )
        .await
        .map_err(|e| {
            format!("Could not connect to Jan: {}", e)
        })?;

        Ok(())
    }

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            "http://127.0.0.1:1337/v1/models"
        )
        .await
        .map_err(|e| {
            format!("Could not connect to Jan: {}", e)
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "Jan returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse Jan response: {}",
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
pub async fn get_jan_models()
    -> Result<Vec<ProviderModel>, String>
{
    let provider = JanProvider::new();

    provider.get_models().await
}