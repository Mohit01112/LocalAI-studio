
use serde::{Deserialize, Serialize};

use super::provider::{
    AIProvider,
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ProviderInfo,
    ProviderModel,
};

const JAN_BASE_URL: &str = "http://127.0.0.1:1337";

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size: String,
}

#[derive(Debug, Deserialize)]
struct JanModelsResponse {
    data: Vec<JanModel>,
}

#[derive(Debug, Deserialize)]
struct JanModel {
    id: String,
}

#[derive(Debug, Serialize)]
struct JanChatRequest {
    model: String,
    messages: Vec<JanMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JanMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct JanChatResponse {
    choices: Vec<JanChoice>,
}

#[derive(Debug, Deserialize)]
struct JanChoice {
    message: JanMessage,
    finish_reason: Option<String>,
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
        let running = reqwest::get(
            format!("{}/v1/models", JAN_BASE_URL)
        )
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false);

        ProviderInfo {
            id: "jan".to_string(),
            name: "Jan".to_string(),
            installed: true,
            running,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        let response = reqwest::get(
            format!("{}/v1/models", JAN_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to Jan: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Jan returned HTTP status: {}",
                response.status()
            ));
        }

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            format!("{}/v1/models", JAN_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to Jan: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Jan returned HTTP status: {}",
                response.status()
            ));
        }

        let data: JanModelsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!("Failed to parse Jan models: {}", e)
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
            return Err("No Jan model was selected.".to_string());
        }

        if request.messages.is_empty() {
            return Err("Chat request contains no messages.".to_string());
        }

        let mut messages: Vec<JanMessage> = request
            .messages
            .iter()
            .map(|message| JanMessage {
                role: message.role.clone(),
                content: message.content.clone(),
            })
            .collect();

        // Attach files to the latest user message.
        // PDFs are decoded and their text is extracted locally.
        // Image support depends on the loaded model and API format.
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
                    "Image attachment '{}' is not supported by this text-only Jan request. Image support depends on the loaded model and API format.",
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

        let body = JanChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            temperature: request.temperature,
        };

        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "{}/v1/chat/completions",
                JAN_BASE_URL
            ))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Failed to send request to Jan: {}",
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
                    "Jan returned HTTP status: {}",
                    status
                ));
            }

            return Err(format!(
                "Jan error ({}): {}",
                status,
                error_text
            ));
        }

        let data: JanChatResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse Jan response: {}",
                    e
                )
            })?;

        let choice = data.choices.into_iter().next()
            .ok_or_else(|| {
                "Jan returned no response choices.".to_string()
            })?;

        // Return the raw content from Jan exactly as received.
        let content = choice.message.content;

        if content.is_empty() {
            return Err(
                "Jan returned an empty response.".to_string()
            );
        }

        Ok(ChatResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
            },
            done: true,
            provider: Some("jan".to_string()),
            model: Some(request.model.clone()),
        })
    }
}

#[tauri::command]
pub async fn get_jan_models() -> Result<Vec<ModelInfo>, String> {
    let provider = JanProvider::new();

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

