use serde::Serialize;
use std::process::Command;
use std::net::TcpStream;

#[derive(Debug, Serialize)]
pub struct RuntimeInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub running: bool,
}

#[tauri::command]
pub fn discover_runtimes() -> Vec<RuntimeInfo> {
    vec![
        detect_ollama(),
        detect_lm_studio(),
        detect_llama_cpp(),
        detect_jan(),
        detect_gpt4all(),
        detect_localai(),
        detect_koboldcpp(),
        detect_llamafile(),
        detect_vllm(),
        detect_mlx(),
    ]
}

// --------------------------------------------------
// Ollama
// --------------------------------------------------

fn detect_ollama() -> RuntimeInfo {
    let installed = command_exists("ollama");
    let running = installed && api_available("http://127.0.0.1:11434/api/tags");

    RuntimeInfo {
        id: "ollama".into(),
        name: "Ollama".into(),
        installed,
        running,
    }
}

// --------------------------------------------------
// LM Studio
// --------------------------------------------------

fn detect_lm_studio() -> RuntimeInfo {
    let running = api_available("http://127.0.0.1:1234/v1/models");

    let process_running =
        process_exists("LM Studio")
            || process_exists("lmstudio")
            || process_exists("lm-studio");

    let installed = running || process_running;

    RuntimeInfo {
        id: "lm-studio".into(),
        name: "LM Studio".into(),
        installed,
        running,
    }
}

// --------------------------------------------------
// llama.cpp
// --------------------------------------------------

fn detect_llama_cpp() -> RuntimeInfo {
    let executable_installed = command_exists("llama-server");

    let running = identify_openai_runtime(
        "http://127.0.0.1:8080/v1/models",
        &[
            "llama",
            "llama.cpp",
        ],
    );

    RuntimeInfo {
        id: "llama-cpp".into(),
        name: "llama.cpp".into(),
        installed: executable_installed || running,
        running,
    }
}

// --------------------------------------------------
// Jan
// --------------------------------------------------

fn detect_jan() -> RuntimeInfo {
    let running = api_available("http://127.0.0.1:1337/v1/models");

    let process_running =
        process_exists("Jan")
            || process_exists("jan");

    RuntimeInfo {
        id: "jan".into(),
        name: "Jan".into(),
        installed: running || process_running,
        running,
    }
}

// --------------------------------------------------
// GPT4All
// --------------------------------------------------

fn detect_gpt4all() -> RuntimeInfo {
    let running = api_available("http://127.0.0.1:4891/v1/models");

    let process_running =
        process_exists("GPT4All")
            || process_exists("gpt4all");

    RuntimeInfo {
        id: "gpt4all".into(),
        name: "GPT4All".into(),
        installed: running || process_running,
        running,
    }
}

// --------------------------------------------------
// LocalAI
// --------------------------------------------------

fn detect_localai() -> RuntimeInfo {
    let running = identify_openai_runtime(
        "http://127.0.0.1:8080/v1/models",
        &[
            "localai",
            "local-ai",
        ],
    );

    let process_running =
        process_exists("local-ai")
            || process_exists("localai");

    RuntimeInfo {
        id: "localai".into(),
        name: "LocalAI".into(),
        installed: running || process_running,
        running,
    }
}

// --------------------------------------------------
// KoboldCpp
// --------------------------------------------------

fn detect_koboldcpp() -> RuntimeInfo {
    let executable_installed = command_exists("koboldcpp");

    let running = api_available(
        "http://127.0.0.1:5001/v1/models",
    );

    let process_running =
        process_exists("koboldcpp")
            || process_exists("KoboldCpp");

    RuntimeInfo {
        id: "koboldcpp".into(),
        name: "KoboldCpp".into(),
        installed: executable_installed || process_running || running,
        running,
    }
}

// --------------------------------------------------
// llamafile
// --------------------------------------------------

fn detect_llamafile() -> RuntimeInfo {
    let process_running =
        process_exists("llamafile");

    RuntimeInfo {
        id: "llamafile".into(),
        name: "llamafile".into(),
        installed: process_running,
        running: false,
    }
}

// --------------------------------------------------
// vLLM
// --------------------------------------------------

fn detect_vllm() -> RuntimeInfo {
    let executable_installed = command_exists("vllm");

    let running = api_available(
        "http://127.0.0.1:8000/v1/models",
    );

    RuntimeInfo {
        id: "vllm".into(),
        name: "vLLM".into(),
        installed: executable_installed || running,
        running,
    }
}

// --------------------------------------------------
// MLX / MLX-LM
// --------------------------------------------------

fn detect_mlx() -> RuntimeInfo {
    if !cfg!(target_os = "macos") {
        return RuntimeInfo {
            id: "mlx".into(),
            name: "MLX / MLX-LM".into(),
            installed: false,
            running: false,
        };
    }

    let mlx_installed =
        command_exists("mlx_lm.generate")
            || command_exists("mlx_lm.server")
            || command_exists("mlx_lm");

    RuntimeInfo {
        id: "mlx".into(),
        name: "MLX / MLX-LM".into(),
        installed: mlx_installed,
        running: false,
    }
}

// --------------------------------------------------
// Helpers
// --------------------------------------------------

fn command_exists(command: &str) -> bool {
    let checker = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    Command::new(checker)
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn process_exists(process_name: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .output()
    } else {
        Command::new("ps")
            .arg("-A")
            .output()
    };

    match output {
        Ok(output) => {
            let stdout =
                String::from_utf8_lossy(&output.stdout);

            stdout
                .to_lowercase()
                .contains(&process_name.to_lowercase())
        }

        Err(_) => false,
    }
}

fn api_available(url: &str) -> bool {
    match reqwest::blocking::get(url) {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

fn identify_openai_runtime(
    url: &str,
    identifiers: &[&str],
) -> bool {
    let response = match reqwest::blocking::get(url) {
        Ok(response) => response,
        Err(_) => return false,
    };

    if !response.status().is_success() {
        return false;
    }

    let body = match response.text() {
        Ok(body) => body.to_lowercase(),
        Err(_) => return false,
    };

    identifiers
        .iter()
        .any(|identifier| body.contains(&identifier.to_lowercase()))
}

#[allow(dead_code)]
fn port_open(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).is_ok()
}