use serde::{Deserialize, Serialize};

use super::provider::{
    AIProvider,
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ProviderInfo,
    ProviderModel,
};

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
}

pub struct OllamaProvider;

impl OllamaProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AIProvider for OllamaProvider {
    async fn info(&self) -> ProviderInfo {
        let running = reqwest::get(
            format!("{}/api/tags", OLLAMA_BASE_URL)
        )
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false);

        ProviderInfo {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            installed: true,
            running,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        let response = reqwest::get(
            format!("{}/api/tags", OLLAMA_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama returned HTTP status: {}",
                response.status()
            ));
        }

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(
            format!("{}/api/tags", OLLAMA_BASE_URL)
        )
        .await
        .map_err(|e| format!("Could not connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama returned HTTP status: {}",
                response.status()
            ));
        }

        let data: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| {
                format!("Failed to parse Ollama models: {}", e)
            })?;

        Ok(data
            .models
            .into_iter()
            .map(|model| ProviderModel {
                id: model.name.clone(),
                name: model.name,
                size: format_size(model.size),
            })
            .collect())
    }

    async fn chat(
        &self,
        request: &ChatRequest,
    ) -> Result<ChatResponse, String> {
        if request.model.trim().is_empty() {
            return Err("No Ollama model was selected.".to_string());
        }

        if request.messages.is_empty() {
            return Err("Chat request contains no messages.".to_string());
        }

        let mut messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|message| OllamaMessage {
                role: message.role.clone(),
                content: message.content.clone(),
                images: Vec::new(),
            })
            .collect();

        // Attach files to the latest user message.
        // Images are sent using Ollama's native `images` field.
        // PDFs are decoded and their text is extracted locally before
        // the request is sent to Ollama.
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
                target.images.push(attachment.data.clone());
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

        let body = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
        };

        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "{}/api/chat",
                OLLAMA_BASE_URL
            ))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Failed to send request to Ollama: {}",
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
                    "Ollama returned HTTP status: {}",
                    status
                ));
            }

            return Err(format!(
                "Ollama error ({}): {}",
                status,
                error_text
            ));
        }

        let data: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| {
                format!(
                    "Failed to parse Ollama response: {}",
                    e
                )
            })?;

        // Return the raw content from Ollama exactly as received, no trimming
        // or modification, so it matches what you see on the terminal.
        let content = data.message.content;

        if content.is_empty() {
            return Err(
                "Ollama returned an empty response.".to_string()
            );
        }

        Ok(ChatResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
            },
            done: data.done,
            provider: Some("ollama".to_string()),
            model: Some(request.model.clone()),
        })
    }
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<ModelInfo>, String> {
    let provider = OllamaProvider::new();

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

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!(
            "{:.2} GB",
            bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        )
    } else {
        format!(
            "{:.0} MB",
            bytes as f64 / (1024.0 * 1024.0)
        )
    }
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
