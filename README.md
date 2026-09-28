# LocalAI Studio

**A desktop application for discovering and chatting with locally hosted
AI models.**

LocalAI Studio helps you detect supported local AI runtimes, view
available models, and start chatting with your local LLMs from one
desktop interface.

> **Version:** 0.1.0\
> **Platform:** Windows\
> **Status:** Initial release

------------------------------------------------------------------------

## Demo

Add your screen recordings to the `docs/videos/` folder in this
repository, then update the filenames below if needed. GitHub renders
MP4 files linked from a README, and these embedded videos let visitors
see the app in action.

### App overview

```{=html}
<!-- Add docs/videos/app-overview.mp4 to the repository -->
```
https://github.com/Mohit01112/LocalAI-studio/blob/main/docs/videos/app-overview.mp4

### Runtime and model detection

```{=html}
<!-- Add docs/videos/runtime-model-detection.mp4 to the repository -->
```
https://github.com/Mohit01112/LocalAI-studio/blob/main/docs/videos/runtime-model-detection.mp4

### Chat with a local model

```{=html}
<!-- Add docs/videos/local-model-chat.mp4 to the repository -->
```
https://github.com/Mohit01112/LocalAI-studio/blob/main/docs/videos/local-model-chat.mp4

**Tip:** For a video to display inline on GitHub, upload it to the
repository and link to its raw file URL. If GitHub does not render the
video inline in your README, use a thumbnail image that links to the MP4
instead.

------------------------------------------------------------------------

## Features

-   **Runtime detection:** Detects supported local AI runtimes installed
    on your machine.
-   **Model discovery:** Retrieves models available through a detected
    runtime.
-   **Local chat:** Provides a desktop interface for interacting with
    supported local models.
-   **Explore runtimes and models:** Offers an exploration path when no
    models are detected.
-   **Desktop app:** Built with Tauri, with a React and TypeScript
    frontend.

### Supported runtimes

The application includes detection support for:

-   Ollama
-   LM Studio
-   llama.cpp
-   Jan
-   GPT4All
-   LocalAI
-   KoboldCpp
-   llamafile
-   vLLM
-   MLX

Runtime and model availability depends on the runtime being installed,
running, and configured on your system.

------------------------------------------------------------------------

## Download and install (Windows)

1.  Open the [GitHub Releases
    page](https://github.com/Mohit01112/LocalAI-studio/releases).
2.  Open the release you want to install.
3.  Under **Assets**, download one of the Windows installers:
    -   **`.exe`** --- NSIS installer
    -   **`.msi`** --- Windows Installer package
4.  Run the downloaded installer and follow the on-screen instructions.
5.  Launch **LocalAI Studio** from the Start menu or desktop shortcut,
    if created.

If the Releases page does not show a published release yet, use the
build-from-source instructions below or publish a release and attach the
installer files.

### Before using local models

Install and start a supported runtime, and make sure at least one model
is available in it. For example, with Ollama:

1.  Install Ollama from [ollama.com](https://ollama.com/).

2.  Start Ollama.

3.  Download a model using the Ollama CLI, for example:

    ``` powershell
    ollama pull llama3.2:3b
    ```

4.  Confirm the model is available:

    ``` powershell
    ollama list
    ```

5.  Open LocalAI Studio and scan/select the runtime.

LocalAI Studio does not bundle model weights. Models are managed by the
runtime and may require substantial disk space and memory.

------------------------------------------------------------------------

## Build from source

### Requirements

-   Windows 10 or later
-   [Node.js](https://nodejs.org/) and npm
-   [Rust](https://www.rust-lang.org/tools/install) toolchain
-   Tauri prerequisites for Windows, including Microsoft C++ Build Tools
    and WebView2

See the [Tauri v2 prerequisites for
Windows](https://v2.tauri.app/start/prerequisites/#windows) for current
setup details.

### Clone the repository

``` powershell
git clone https://github.com/Mohit01112/LocalAI-studio.git
cd LocalAI-studio
```

### Install dependencies

``` powershell
npm install
```

### Run in development mode

``` powershell
npx tauri dev
```

### Build the frontend

``` powershell
npm run build
```

### Build the Windows desktop installers

``` powershell
npx tauri build
```

The generated installers are typically located under:

``` text
src-tauri/target/release/bundle/
```

Depending on the Tauri bundler configuration, you may see an `.msi`
installer and an NSIS `.exe` installer in their respective subfolders.

------------------------------------------------------------------------

## Tech stack

-   **Frontend:** React, TypeScript, Vite
-   **Desktop framework:** Tauri
-   **Backend:** Rust
-   **Runtime communication:** Local runtime APIs and system runtime
    detection

------------------------------------------------------------------------

## Repository structure

``` text
LocalAI-studio/
├── src/                 # React frontend
├── src-tauri/           # Tauri and Rust backend
├── public/              # Static frontend assets (if present)
├── package.json
└── README.md
```

The exact files and folders may evolve as the project develops.

------------------------------------------------------------------------

## Troubleshooting

### Ollama is installed but no models appear

-   Make sure Ollama is running.
-   Run `ollama list` in PowerShell to confirm that a model is
    installed.
-   Check that the Ollama API is reachable at `http://127.0.0.1:11434`.
-   Restart LocalAI Studio after starting the runtime.

### The app cannot connect to a runtime

-   Confirm the runtime is installed and running.
-   Check the runtime's own logs and API endpoint.
-   Make sure local firewall or security software is not blocking the
    connection.

### The installer is blocked by Windows

Windows may show a security warning for an unsigned or self-distributed
installer. Only install builds from a source you trust. Code signing can
reduce these warnings for future releases.

------------------------------------------------------------------------

## Roadmap

-   Improve runtime compatibility and error reporting
-   Refine model discovery and connection handling
-   Add more polished onboarding and documentation
-   Expand testing across supported runtimes

------------------------------------------------------------------------

## Contributing

Issues and pull requests are welcome. Please include your Windows
version, runtime name/version, and relevant error details when reporting
a problem. Do not include API keys, passwords, or other secrets in issue
reports.

------------------------------------------------------------------------

## License

No license has been specified yet. Until a license is added to the
repository, reuse, modification, and redistribution are not explicitly
granted.

------------------------------------------------------------------------

## Author

**Mohit Jadhav**

-   GitHub: [Mohit01112](https://github.com/Mohit01112)
-   Repository: [LocalAI
    Studio](https://github.com/Mohit01112/LocalAI-studio)
