use super::{
    gpt4all::GPT4AllProvider,
    jan::JanProvider,
    koboldcpp::KoboldCppProvider,
    llama_cpp::LlamaCppProvider,
    llamafile::LlamafileProvider,
    lm_studio::LMStudioProvider,
    localai::LocalAIProvider,
    mlx::MlxProvider,
    ollama::OllamaProvider,
    provider::{
        AIProvider,
        ChatRequest,
        ChatResponse,
        ProviderInfo,
        ProviderModel,
    },
    vllm::VllmProvider,
};

pub struct ProviderManager;

impl ProviderManager {
    pub fn new() -> Self {
        Self
    }

    pub fn provider(
        &self,
        runtime_id: &str,
    ) -> Result<Box<dyn AIProvider + Send + Sync>, String> {
        match runtime_id {
            "ollama" => Ok(Box::new(OllamaProvider::new())),
            "lm-studio" => Ok(Box::new(LMStudioProvider::new())),
            "llama-cpp" => Ok(Box::new(LlamaCppProvider::new())),
            "jan" => Ok(Box::new(JanProvider::new())),
            "gpt4all" => Ok(Box::new(GPT4AllProvider::new())),
            "localai" => Ok(Box::new(LocalAIProvider::new())),
            "koboldcpp" => Ok(Box::new(KoboldCppProvider::new())),
            "llamafile" => Ok(Box::new(LlamafileProvider::new())),
            "vllm" => Ok(Box::new(VllmProvider::new())),
            "mlx" => Ok(Box::new(MlxProvider::new())),
            _ => Err(format!(
                "Unknown AI runtime: {}",
                runtime_id
            )),
        }
    }

    pub async fn get_info(
        &self,
        runtime_id: &str,
    ) -> Result<ProviderInfo, String> {
        let provider = self.provider(runtime_id)?;

        Ok(provider.info().await)
    }

    pub async fn connect(
        &self,
        runtime_id: &str,
    ) -> Result<(), String> {
        let provider = self.provider(runtime_id)?;

        provider.connect().await
    }

    pub async fn get_models(
        &self,
        runtime_id: &str,
    ) -> Result<Vec<ProviderModel>, String> {
        let provider = self.provider(runtime_id)?;

        provider.get_models().await
    }

    pub async fn start(
        &self,
        runtime_id: &str,
    ) -> Result<(), String> {
        let provider = self.provider(runtime_id)?;

        provider.start().await
    }

    pub async fn stop(
        &self,
        runtime_id: &str,
    ) -> Result<(), String> {
        let provider = self.provider(runtime_id)?;

        provider.stop().await
    }

    pub async fn download_model(
        &self,
        runtime_id: &str,
        model: &str,
    ) -> Result<(), String> {
        let provider = self.provider(runtime_id)?;

        provider.download_model(model).await
    }

    pub async fn delete_model(
        &self,
        runtime_id: &str,
        model: &str,
    ) -> Result<(), String> {
        let provider = self.provider(runtime_id)?;

        provider.delete_model(model).await
    }

    /// Sends a provider-neutral chat request to the selected runtime.
    ///
    /// The ProviderManager does not know how a runtime handles
    /// chat internally. That responsibility belongs to the
    /// individual provider implementation.
    pub async fn chat(
        &self,
        runtime_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponse, String> {
        let provider = self.provider(runtime_id)?;

        provider.chat(&request).await
    }
}

#[tauri::command]
pub async fn provider_get_info(
    runtime_id: String,
) -> Result<ProviderInfo, String> {
    ProviderManager::new()
        .get_info(&runtime_id)
        .await
}

#[tauri::command]
pub async fn provider_connect(
    runtime_id: String,
) -> Result<(), String> {
    ProviderManager::new()
        .connect(&runtime_id)
        .await
}

#[tauri::command]
pub async fn provider_get_models(
    runtime_id: String,
) -> Result<Vec<ProviderModel>, String> {
    ProviderManager::new()
        .get_models(&runtime_id)
        .await
}

#[tauri::command]
pub async fn provider_start(
    runtime_id: String,
) -> Result<(), String> {
    ProviderManager::new()
        .start(&runtime_id)
        .await
}

#[tauri::command]
pub async fn provider_stop(
    runtime_id: String,
) -> Result<(), String> {
    ProviderManager::new()
        .stop(&runtime_id)
        .await
}

#[tauri::command]
pub async fn provider_download_model(
    runtime_id: String,
    model: String,
) -> Result<(), String> {
    ProviderManager::new()
        .download_model(&runtime_id, &model)
        .await
}

#[tauri::command]
pub async fn provider_delete_model(
    runtime_id: String,
    model: String,
) -> Result<(), String> {
    ProviderManager::new()
        .delete_model(&runtime_id, &model)
        .await
}

/// Generic chat command exposed to the React frontend.
///
/// React only needs to provide:
/// - runtime_id
/// - ChatRequest
///
/// It does not need to know the native API of Ollama,
/// LM Studio, llama.cpp, Jan, vLLM, etc.
#[tauri::command]
pub async fn provider_chat(
    runtime_id: String,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    ProviderManager::new()
        .chat(&runtime_id, request)
        .await
}