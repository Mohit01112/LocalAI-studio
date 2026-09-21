use serde::Deserialize;

use super::provider::{
    AIProvider,
    ProviderInfo,
    ProviderModel,
};

#[derive(Debug, Deserialize)]
struct KoboldModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<KoboldModel>,
}

pub struct KoboldCppProvider;

impl KoboldCppProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for KoboldCppProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "koboldcpp".to_string(),
            name: "KoboldCpp".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        reqwest::get(
            "http://127.0.0.1:5001/v1/models",
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to KoboldCpp: {}",
                e
            )
        })?;

        Ok(())
    }

    async fn get_models(
        &self,
    ) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            "http://127.0.0.1:5001/v1/models",
        )
        .await
        .map_err(|e| {
            format!(
                "Could not connect to KoboldCpp: {}",
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "KoboldCpp returned HTTP status: {}",
                response.status()
            ));
        }

        let data: ModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse KoboldCpp response: {}",
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
pub async fn get_koboldcpp_models()
    -> Result<Vec<ProviderModel>, String>
{
    let provider = KoboldCppProvider::new();

    provider.get_models().await
}