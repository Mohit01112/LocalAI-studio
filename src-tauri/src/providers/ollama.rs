use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

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
    digest: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

pub struct OllamaProvider;

impl OllamaProvider {
    pub fn new() -> Self {
        Self
    }

    /// Starts the real Ollama CLI process:
    ///
    ///     ollama run <model>
    ///
    /// The user's question is written directly into the
    /// process stdin, exactly like typing into the Ollama
    /// terminal session.
    fn run_ollama_cli(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let mut child = Command::new("ollama")
            .arg("run")
            .arg(model)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "Could not start Ollama. Make sure Ollama is installed and available in PATH: {}",
                    e
                )
            })?;

        // Send the user's question directly to Ollama's stdin.
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| {
                    format!("Failed to send question to Ollama: {}", e)
                })?;

            stdin
                .write_all(b"\n")
                .map_err(|e| {
                    format!("Failed to finish Ollama input: {}", e)
                })?;

            // Closing stdin tells Ollama that this request is complete.
            drop(stdin);
        }

        let output = child
            .wait_with_output()
            .map_err(|e| {
                format!(
                    "Failed while waiting for Ollama response: {}",
                    e
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string();

            if stderr.is_empty() {
                return Err(format!(
                    "Ollama exited with status: {}",
                    output.status
                ));
            }

            return Err(format!("Ollama error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let response = clean_ollama_output(&stdout);

        if response.trim().is_empty() {
            return Err(
                "Ollama returned an empty response.".to_string()
            );
        }

        Ok(response)
    }
}

#[async_trait::async_trait]
impl AIProvider for OllamaProvider {
    async fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            installed: true,
            running: true,
        }
    }

    async fn connect(&self) -> Result<(), String> {
        let response = reqwest::get(format!("{}/api/tags", OLLAMA_BASE_URL))
            .await
            .map_err(|e| {
                format!("Could not connect to Ollama: {}", e)
            })?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama returned HTTP status: {}",
                response.status()
            ));
        }

        Ok(())
    }

    async fn get_models(&self) -> Result<Vec<ProviderModel>, String> {
        let response = reqwest::get(format!("{}/api/tags", OLLAMA_BASE_URL))
            .await
            .map_err(|e| {
                format!("Could not connect to Ollama: {}", e)
            })?;

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
                format!(
                    "Failed to parse Ollama response: {}",
                    e
                )
            })?;

        Ok(data
            .models
            .into_iter()
            .map(|model| {
                let _digest = model.digest;

                ProviderModel {
                    id: model.name.clone(),
                    name: model.name,
                    size: format_size(model.size),
                }
            })
            .collect())
    }

    async fn chat(
        &self,
        request: &ChatRequest,
    ) -> Result<ChatResponse, String> {
        if request.model.trim().is_empty() {
            return Err(
                "No Ollama model was selected.".to_string()
            );
        }

        if request.messages.is_empty() {
            return Err(
                "Chat request contains no messages.".to_string()
            );
        }

        // Only send the latest user question.
        //
        // This is intentional because you asked for:
        //
        // App question
        //      ↓
        // Ollama CMD
        //      ↓
        // CMD output
        //      ↓
        // App
        //
        let prompt = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content.trim())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| {
                "No user message was found.".to_string()
            })?;

        let response = self
            .run_ollama_cli(&request.model, prompt)?;

        Ok(ChatResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content: response,
            },
            done: true,
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

/// Removes terminal control sequences that Ollama can produce
/// when its output is captured from a subprocess.
fn clean_ollama_output(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some('[') = chars.peek() {
                chars.next();

                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }

            continue;
        }

        if ch == '\r' {
            continue;
        }

        output.push(ch);
    }

    output.trim().to_string()
}