import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface RuntimeInfo {
  id: string;
  name: string;
  installed: boolean;
  running: boolean;
}

interface ModelInfo {
  id: string;
  name: string;
  size: string;
}

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

function App() {
  const [runtimes, setRuntimes] = useState<RuntimeInfo[]>([]);
  const [models, setModels] = useState<ModelInfo[]>([]);

  const [selectedRuntime, setSelectedRuntime] = useState("");
  const [selectedModel, setSelectedModel] = useState("");

  const [loadingRuntimes, setLoadingRuntimes] = useState(true);
  const [loadingModels, setLoadingModels] = useState(false);

  const [error, setError] = useState("");
  const [showPlayground, setShowPlayground] = useState(false);

  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    discoverRuntimes();
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({
      behavior: "smooth",
    });
  }, [messages, sending]);

  async function discoverRuntimes() {
    try {
      setLoadingRuntimes(true);
      setError("");

      const result = await invoke<RuntimeInfo[]>(
        "discover_runtimes"
      );

      setRuntimes(result);
    } catch (err) {
      console.error("Runtime discovery error:", err);
      setError("Failed to detect AI runtimes.");
    } finally {
      setLoadingRuntimes(false);
    }
  }

  async function selectRuntime(runtimeId: string) {
    setSelectedRuntime(runtimeId);
    setSelectedModel("");
    setModels([]);
    setShowPlayground(false);
    setMessages([]);
    setError("");

    if (!runtimeId) {
      return;
    }

    try {
      setLoadingModels(true);

      const result = await invoke<ModelInfo[]>(
        "provider_get_models",
        {
          runtimeId,
        }
      );

      setModels(result);
    } catch (err) {
      console.error("Model detection error:", err);

      const runtime = runtimes.find(
        (item) => item.id === runtimeId
      );

      setError(
        `Could not connect to ${
          runtime?.name ?? "the selected runtime"
        }.`
      );
    } finally {
      setLoadingModels(false);
    }
  }

  function continueToPlayground() {
    if (!selectedRuntime || !selectedModel) {
      return;
    }

    setError("");
    setMessages([]);
    setShowPlayground(true);
  }

  function goBack() {
    setShowPlayground(false);
    setMessages([]);
    setInput("");
    setError("");
  }

  function clearChat() {
    setMessages([]);
    setError("");
  }

async function sendMessage() {
  const message = input.trim();

  if (
    !message ||
    !selectedRuntime ||
    !selectedModel ||
    sending
  ) {
    return;
  }

  const userMessage: ChatMessage = {
    role: "user",
    content: message,
  };

  // Send only the actual conversation to the local runtime.
  const conversation = [
    ...messages,
    userMessage,
  ];

  setMessages(conversation);
  setInput("");
  setError("");
  setSending(true);

  try {
    const response = await invoke<{
      message: {
        role: string;
        content: string;
      };
      done: boolean;
      provider?: string;
      model?: string;
    }>("provider_chat", {
      runtimeId: selectedRuntime,
      request: {
        model: selectedModel,
        messages: conversation,
        stream: false,
      },
    });

    const assistantMessage: ChatMessage = {
      role: "assistant",
      content: response.message.content,
    };

    setMessages((previous) => [
      ...previous,
      assistantMessage,
    ]);
  } catch (err) {
    console.error("Chat error:", err);

    setError(
      typeof err === "string"
        ? err
        : "Failed to get a response from the local AI model."
    );
  } finally {
    setSending(false);
  }
}

  function handleInputKeyDown(
    event: React.KeyboardEvent<HTMLTextAreaElement>
  ) {
    if (
      event.key === "Enter" &&
      !event.shiftKey
    ) {
      event.preventDefault();
      sendMessage();
    }
  }

  const selectedRuntimeInfo = runtimes.find(
    (runtime) => runtime.id === selectedRuntime
  );

  const selectedModelInfo = models.find(
    (model) => model.id === selectedModel
  );

  /*
   * ---------------------------------------------------------
   * PLAYGROUND
   * ---------------------------------------------------------
   */

  if (showPlayground) {
    return (
      <div className="playground">

        <header className="playground-header">
          <div className="playground-brand">
            <div className="logo small-logo">
              ◉
            </div>

            <div>
              <h2>LocalAI Studio</h2>
              <span>Local AI Playground</span>
            </div>
          </div>

          <div className="playground-runtime">
            <span className="runtime-dot">
              ●
            </span>

            <span>
              {selectedRuntimeInfo?.name}
            </span>

            <span className="header-separator">
              /
            </span>

            <span>
              {selectedModelInfo?.name}
            </span>
          </div>

          <div className="playground-actions">
            <button
              className="header-button"
              onClick={clearChat}
              disabled={messages.length === 0}
            >
              New Chat
            </button>

            <button
              className="header-button"
              onClick={goBack}
            >
              ← Back
            </button>
          </div>
        </header>

        <main className="chat-container">

          <div className="chat-messages">

            {messages.length === 0 && (
              <div className="chat-welcome">

                <div className="chat-welcome-icon">
                  ◉
                </div>

                <h1>
                  Local AI Playground
                </h1>

                <p>
                  Start a conversation with your
                  local model.
                </p>

                <div className="model-badge">
                  <span>
                    {selectedRuntimeInfo?.name}
                  </span>

                  <span>
                    •
                  </span>

                  <span>
                    {selectedModelInfo?.name}
                  </span>
                </div>

              </div>
            )}

            {messages.map((message, index) => (
              <div
                key={index}
                className={`message-row ${message.role}`}
              >
                <div className="message-avatar">
                  {message.role === "user"
                    ? "You"
                    : "AI"}
                </div>

                <div className="message-content">
                  <div className="message-role">
                    {message.role === "user"
                      ? "You"
                      : "Local AI"}
                  </div>

                  <div className="message-text">
                    {message.content}
                  </div>
                </div>
              </div>
            ))}

            {sending && (
              <div className="message-row assistant">
                <div className="message-avatar">
                  AI
                </div>

                <div className="message-content">
                  <div className="message-role">
                    Local AI
                  </div>

                  <div className="typing-indicator">
                    <span></span>
                    <span></span>
                    <span></span>
                  </div>
                </div>
              </div>
            )}

            <div ref={messagesEndRef} />

          </div>

          {error && (
            <div className="chat-error">
              {error}
            </div>
          )}

          <div className="chat-input-wrapper">

            <div className="chat-input-box">

              <textarea
                value={input}
                onChange={(event) =>
                  setInput(event.target.value)
                }
                onKeyDown={handleInputKeyDown}
                placeholder="Ask your local AI..."
                disabled={sending}
                rows={1}
              />

              <button
                className="send-button"
                onClick={sendMessage}
                disabled={
                  !input.trim() ||
                  sending
                }
              >
                {sending ? "..." : "Send"}
              </button>

            </div>

            <div className="input-hint">
              Press Enter to send • Shift + Enter
              for a new line
            </div>

          </div>

        </main>

      </div>
    );
  }

  /*
   * ---------------------------------------------------------
   * SETUP SCREEN
   * ---------------------------------------------------------
   */

  return (
    <div className="app">
      <main className="welcome-card">

        <div className="logo">
          ◉
        </div>

        <h1>
          Welcome to LocalAI Studio
        </h1>

        <p className="subtitle">
          Discover and manage your local AI models.
        </p>

        <section className="setup-section">

          <label>
            Detected Local AI Runtimes
          </label>

          {loadingRuntimes ? (
            <div className="status-box">
              Detecting local AI runtimes...
            </div>
          ) : runtimes.filter(
              (runtime) => runtime.installed
            ).length > 0 ? (
            <select
              value={selectedRuntime}
              onChange={(event) =>
                selectRuntime(
                  event.target.value
                )
              }
            >
              <option value="">
                Select a runtime
              </option>

              {runtimes
                .filter(
                  (runtime) =>
                    runtime.installed
                )
                .map((runtime) => (
                  <option
                    key={runtime.id}
                    value={runtime.id}
                  >
                    {runtime.name}

                    {runtime.running
                      ? " • Running"
                      : " • Installed"}
                  </option>
                ))}
            </select>
          ) : (
            <div className="status-box">
              No local AI runtime detected.
            </div>
          )}

        </section>

        <section className="setup-section">

          <label>
            Detected Models
          </label>

          {loadingModels ? (
            <div className="status-box">
              Detecting models...
            </div>
          ) : models.length > 0 ? (
            <select
              value={selectedModel}
              onChange={(event) =>
                setSelectedModel(
                  event.target.value
                )
              }
            >
              <option value="">
                Select a model
              </option>

              {models.map((model) => (
                <option
                  key={model.id}
                  value={model.name}
                >
                  {model.name}

                  {model.size !== "Unknown"
                    ? ` • ${model.size}`
                    : ""}
                </option>
              ))}
            </select>
          ) : selectedRuntime ? (
            <div className="model-empty">

              <p>
                No models / LLM detected.
              </p>

              <button className="secondary-button">
                Download LLM / Model
              </button>

            </div>
          ) : (
            <select disabled>
              <option>
                Select a runtime first
              </option>
            </select>
          )}

        </section>

        {error && (
          <div className="error-box">
            {error}
          </div>
        )}

        {selectedRuntimeInfo && (
          <div className="runtime-status">

            <span>
              Runtime:{" "}
              <strong>
                {selectedRuntimeInfo.name}
              </strong>
            </span>

            <span>
              {selectedRuntimeInfo.running
                ? "● Running"
                : "● Installed"}
            </span>

          </div>
        )}

        <button
          className="continue-button"
          disabled={
            !selectedRuntime ||
            !selectedModel
          }
          onClick={continueToPlayground}
        >
          Continue
        </button>

      </main>
    </div>
  );
}

export default App;