
use serde::{Deserialize, Serialize};

use super::provider::{
    AIProvider,
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ProviderInfo,
    ProviderModel,
};

const LM_STUDIO_BASE_URL: &str = "http://127.0.0.1:1234";

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size: String,
}

#[derive(Debug, Deserialize)]
struct LMStudioModelsResponse {
    data: Vec<LMStudioModel>,
}

#[derive(Debug, Deserialize)]
struct LMStudioModel {
    id: String,
}

#[derive(Debug, Serialize)]
struct LMStudioChatRequest {
    model: String,
    messages: Vec<LMStudioMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LMStudioMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LMStudioChatResponse {
    choices: Vec<LMStudioChoice>,
}

#[derive(Debug, Deserialize)]
struct LMStudioChoice {
    message: LMStudioMessage,
    #[serde(rename = "finish_reason")]
    _finish_reason: Option<String>,
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
        let running = reqwest::get(
            format!("{}/v1/models", LM_STUDIO_BASE_URL)
        )
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false);

        ProviderInfo {
            id: "lm_studio".to_string(),
            name: "LM Studio".to_string(),
            installed: true,
            running,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        let response = reqwest::get(
            format!("{}/v1/models", LM_STUDIO_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to LM Studio: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "LM Studio returned HTTP status: {}",
                response.status()
            ));
        }

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            format!("{}/v1/models", LM_STUDIO_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to LM Studio: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "LM Studio returned HTTP status: {}",
                response.status()
            ));
        }

        let data: LMStudioModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!("Failed to parse LM Studio models: {}", e)
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

    async fn chat(
        &self,
        request: &ChatRequest,
    ) -> Result<ChatResponse, String> {
        if request.model.trim().is_empty() {
            return Err("No LM Studio model was selected.".to_string());
        }

        if request.messages.is_empty() {
            return Err("Chat request contains no messages.".to_string());
        }

        let mut messages: Vec<LMStudioMessage> = request
            .messages
            .iter()
            .map(|message| LMStudioMessage {
                role: message.role.clone(),
                content: message.content.clone(),
            })
            .collect();

        // Attach files to the latest user message.
        // Images are sent using OpenAI-compatible image content.
        // PDFs are decoded and their text is extracted locally.
        for attachment in &request.attachments {
            let target = messages
                .iter_mut()
                .rev()
                .find(|message| message.role == "user")
                .ok_or_else(|| {
                    "File attachment requires a user message.".to_string()
                })?;

            if attachment.kind == "image"
                || attachment.mime_type.starts_with("image/")
            {
                return Err(format!(
                    "Image attachment '{}' is not supported by this LM Studio text request. Image support depends on the loaded model and API format.",
                    attachment.name
                ));
            } else if attachment.kind == "pdf"
                || attachment.mime_type == "application/pdf"
            {
                let pdf_bytes = decode_base64(&attachment.data)
                    .map_err(|e| {
                        format!(
                            "Could not decode PDF '{}': {}",
                            attachment.name,
                            e
                        )
                    })?;

                let pdf_text =
                    pdf_extract::extract_text_from_mem(&pdf_bytes)
                        .map_err(|e| {
                            format!(
                                "Could not extract text from PDF '{}': {}",
                                attachment.name,
                                e
                            )
                        })?;

                if pdf_text.trim().is_empty() {
                    return Err(format!(
                        "PDF '{}' does not contain extractable text. It may be a scanned/image-only PDF.",
                        attachment.name
                    ));
                }

                target.content.push_str(
                    &format!(
                        "\n\n--- Attached PDF: {} ---\n\n{}\n\n--- End Attached PDF ---",
                        attachment.name,
                        pdf_text
                    )
                );
            }
        }

        let body = LMStudioChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            temperature: request.temperature,
        };

        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "{}/v1/chat/completions",
                LM_STUDIO_BASE_URL
            ))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Failed to send request to LM Studio: {}",
                    e
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();

            let error_text = response
                .text()
                .await
                .unwrap_or_default();

            if error_text.trim().is_empty() {
                return Err(format!(
                    "LM Studio returned HTTP status: {}",
                    status
                ));
            }

            return Err(format!(
                "LM Studio error ({}): {}",
                status,
                error_text
            ));
        }

        let data: LMStudioChatResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse LM Studio response: {}",
                    e
                )
            })?;

        let choice = data.choices.into_iter().next()
            .ok_or_else(|| {
                "LM Studio returned no response choices.".to_string()
            })?;

        // Return the raw content from LM Studio exactly as received.
        let content = choice.message.content;

        if content.is_empty() {
            return Err(
                "LM Studio returned an empty response.".to_string()
            );
        }

        Ok(ChatResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
            },
            done: true,
            provider: Some("lm_studio".to_string()),
            model: Some(request.model.clone()),
        })
    }
}

#[tauri::command]
pub async fn get_lm_studio_models() -> Result<Vec<ModelInfo>, String> {
    let provider = LMStudioProvider::new();

    let models = provider.get_models().await?;

    Ok(models
        .into_iter()
        .map(|model| ModelInfo {
            id: model.id,
            name: model.name,
            size: model.size,
        })
        .collect())
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }

        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'\r' | b'\n' | b' ' | b'\t' => continue,
            _ => {
                return Err(
                    "Invalid Base64 data.".to_string()
                )
            }
        };

        buffer = (buffer << 6) | value as u32;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push(
                ((buffer >> bits) & 0xff) as u8
            );
        }
    }

    Ok(output)
}