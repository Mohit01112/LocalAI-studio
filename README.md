# ⚡ LocalAI Studio

### Your local AI workspace. One desktop app. Multiple AI engines.

LocalAI Studio is a desktop application designed to help you discover and interact with locally hosted AI models through a single, modern interface. Built with React, TypeScript, Tauri, and Rust, it brings multiple AI runtimes together in one convenient workspace.

## ✨ Features

- **Desktop Application:** A native desktop experience powered by Tauri.
- **Modern UI:** Built with React, TypeScript, and Vite.
- **Multi-Engine Detection:** Detects several local AI runtimes.
- **Explore AI Engines:** Discover supported engines when none are detected.
- **Local AI Workspace:** Access compatible local AI models through one interface.
- **Native Backend:** Rust handles desktop functionality and runtime detection.
- **Windows Installers:** MSI and EXE installer packages.

## 🔌 Supported AI Runtimes

LocalAI Studio includes runtime detection for:

- Ollama
- LM Studio
- llama.cpp
- Jan
- GPT4All
- LocalAI
- KoboldCpp
- llamafile
- vLLM
- MLX

*Runtime detection does not guarantee that every runtime supports all application features.*

## 🛠️ Tech Stack

| Component | Technology |
|---|---|
| Frontend | React |
| Language | TypeScript |
| Build Tool | Vite |
| Desktop Framework | Tauri |
| Backend | Rust |
| Package Manager | npm |
| Windows Packaging | MSI and NSIS |

## 🏗️ Architecture

```text
             LocalAI Studio
                    |
        React + TypeScript UI
                    |
              Tauri Bridge
                    |
              Rust Backend
                    |
          Runtime Detection
                    |
       Local AI Engines / Models
                    |
   Ollama, LM Studio, llama.cpp,
       Jan, GPT4All and more
