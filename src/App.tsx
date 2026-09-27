
import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
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

interface RecentChat {
  id: number;
  title: string;
  messages: ChatMessage[];
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
  const [isListening, setIsListening] = useState(false);

  const [recentChats, setRecentChats] = useState<RecentChat[]>([]);
  const [activeChatId, setActiveChatId] = useState<number | null>(
    null
  );

  const [copiedCode, setCopiedCode] = useState<number | null>(
    null
  );

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const recognitionRef = useRef<any>(null);

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
    setActiveChatId(null);
    setShowPlayground(true);
  }

  function goBack() {
    setShowPlayground(false);
    setMessages([]);
    setInput("");
    setError("");
    stopVoiceInput();
  }

  function createNewChat() {
    setMessages([]);
    setInput("");
    setError("");
    setActiveChatId(null);
    stopVoiceInput();

    setTimeout(() => {
      textareaRef.current?.focus();
    }, 50);
  }

  function openRecentChat(chat: RecentChat) {
    setActiveChatId(chat.id);
    setMessages(chat.messages);
    setInput("");
    setError("");
    stopVoiceInput();
  }

  function saveCurrentChat(
    updatedMessages: ChatMessage[]
  ) {
    if (updatedMessages.length === 0) {
      return;
    }

    const firstUserMessage = updatedMessages.find(
      (message) => message.role === "user"
    );

    if (!firstUserMessage) {
      return;
    }

    const title =
      firstUserMessage.content.length > 40
        ? `${firstUserMessage.content.slice(0, 40)}...`
        : firstUserMessage.content;

    setRecentChats((previous) => {
      if (activeChatId !== null) {
        return previous.map((chat) =>
          chat.id === activeChatId
            ? {
                ...chat,
                title,
                messages: updatedMessages,
              }
            : chat
        );
      }

      const newChat: RecentChat = {
        id: Date.now(),
        title,
        messages: updatedMessages,
      };

      setActiveChatId(newChat.id);

      return [newChat, ...previous];
    });
  }

  function stopVoiceInput() {
    recognitionRef.current?.stop();
    setIsListening(false);
  }

  function toggleVoiceInput() {
    if (isListening) {
      stopVoiceInput();
      return;
    }

    const SpeechRecognition =
      (window as any).SpeechRecognition ||
      (window as any).webkitSpeechRecognition;

    if (!SpeechRecognition) {
      setError(
        "Voice input is not supported in this WebView. Try a supported browser or enable speech recognition."
      );
      return;
    }

    setError("");

    const recognition = new SpeechRecognition();
    recognition.lang = navigator.language || "en-US";
    recognition.continuous = false;
    recognition.interimResults = true;

    recognition.onstart = () => {
      setIsListening(true);
    };

    recognition.onresult = (event: any) => {
      let transcript = "";

      for (
        let i = event.resultIndex;
        i < event.results.length;
        i += 1
      ) {
        transcript += event.results[i][0].transcript;
      }

      if (transcript) {
        setInput((previous) => {
          const separator =
            previous && !previous.endsWith(" ") ? " " : "";
          return `${previous}${separator}${transcript}`;
        });
      }
    };

    recognition.onerror = (event: any) => {
      setIsListening(false);

      if (
        event.error === "not-allowed" ||
        event.error === "service-not-allowed"
      ) {
        setError(
          "Microphone permission was denied. Allow microphone access and try again."
        );
      } else if (
        event.error !== "no-speech" &&
        event.error !== "aborted"
      ) {
        setError(`Voice input error: ${event.error}`);
      }
    };

    recognition.onend = () => {
      setIsListening(false);
      recognitionRef.current = null;
      textareaRef.current?.focus();
    };

    recognitionRef.current = recognition;

    try {
      recognition.start();
    } catch (err) {
      recognitionRef.current = null;
      setIsListening(false);
      setError("Could not start voice input. Please try again.");
      console.error("Voice input start error:", err);
    }
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

    if (isListening) {
      stopVoiceInput();
    }

    const userMessage: ChatMessage = {
      role: "user",
      content: message,
    };

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

      const updatedMessages = [
        ...conversation,
        assistantMessage,
      ];

      setMessages(updatedMessages);
      saveCurrentChat(updatedMessages);
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
    event: KeyboardEvent<HTMLTextAreaElement>
  ) {
    if (
      event.key === "Enter" &&
      !event.shiftKey
    ) {
      event.preventDefault();
      sendMessage();
    }
  }

  async function copyCode(
    code: string,
    index: number
  ) {
    try {
      await navigator.clipboard.writeText(code);

      setCopiedCode(index);

      setTimeout(() => {
        setCopiedCode(null);
      }, 2000);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  }

  function MarkdownMessage({
    content,
    messageIndex,
  }: {
    content: string;
    messageIndex: number;
  }) {
    return (
      <div className="markdown-content">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={{
            code({
              inline,
              className,
              children,
              ...props
            }: any) {
              const match =
                /language-(\w+)/.exec(
                  className || ""
                );

              const code = String(children).replace(
                /\n$/,
                ""
              );

              if (inline) {
                return (
                  <code
                    className="inline-code"
                    {...props}
                  >
                    {children}
                  </code>
                );
              }

              return (
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-language">
                      {match
                        ? match[1]
                        : "code"}
                    </span>

                    <button
                      className="copy-code-button"
                      onClick={() =>
                        copyCode(
                          code,
                          messageIndex
                        )
                      }
                    >
                      {copiedCode ===
                      messageIndex
                        ? "✓ Copied"
                        : "Copy"}
                    </button>
                  </div>

                  <pre>
                    <code
                      className={
                        className || ""
                      }
                      {...props}
                    >
                      {children}
                    </code>
                  </pre>
                </div>
              );
            },

            p({ children }) {
              return <p>{children}</p>;
            },

            h1({ children }) {
              return <h1>{children}</h1>;
            },

            h2({ children }) {
              return <h2>{children}</h2>;
            },

            h3({ children }) {
              return <h3>{children}</h3>;
            },

            ul({ children }) {
              return <ul>{children}</ul>;
            },

            ol({ children }) {
              return <ol>{children}</ol>;
            },

            li({ children }) {
              return <li>{children}</li>;
            },

            blockquote({ children }) {
              return (
                <blockquote>
                  {children}
                </blockquote>
              );
            },

            table({ children }) {
              return (
                <div className="markdown-table-wrapper">
                  <table>
                    {children}
                  </table>
                </div>
              );
            },

            th({ children }) {
              return <th>{children}</th>;
            },

            td({ children }) {
              return <td>{children}</td>;
            },
          }}
        >
          {content}
        </ReactMarkdown>
      </div>
    );
  }

  const selectedRuntimeInfo = runtimes.find(
    (runtime) =>
      runtime.id === selectedRuntime
  );

  const selectedModelInfo = models.find(
    (model) =>
      model.name === selectedModel
  );

  /*
   * ---------------------------------------------------------
   * CHAT PLAYGROUND
   * ---------------------------------------------------------
   */

  if (showPlayground) {
    return (
      <div className="chat-layout">
        {/* SIDEBAR */}
        <aside className="chat-sidebar">
          <div className="sidebar-top">
            <div className="sidebar-brand">
              <div className="sidebar-logo">
                ◉
              </div>

              <div>
                <strong>
                  LocalAI Studio
                </strong>

                <span>
                  Local AI
                </span>
              </div>
            </div>

            <button
              className="new-chat-button"
              onClick={createNewChat}
            >
              <span className="new-chat-icon">
                +
              </span>

              <span>
                New chat
              </span>
            </button>

            <div className="sidebar-section-title">
              Recent
            </div>

            <div className="recent-chats">
              {recentChats.length === 0 ? (
                <div className="no-recent-chats">
                  No recent chats
                </div>
              ) : (
                recentChats.map((chat) => (
                  <button
                    key={chat.id}
                    className={`recent-chat ${
                      activeChatId === chat.id
                        ? "active"
                        : ""
                    }`}
                    onClick={() =>
                      openRecentChat(chat)
                    }
                  >
                    <span className="recent-chat-icon">
                      ◌
                    </span>

                    <span className="recent-chat-title">
                      {chat.title}
                    </span>
                  </button>
                ))
              )}
            </div>
          </div>

          <div className="sidebar-bottom">
            <div className="sidebar-model">
              <span className="sidebar-status-dot"></span>

              <div>
                <small>
                  {selectedRuntimeInfo?.name}
                </small>

                <strong>
                  {selectedModelInfo?.name ??
                    selectedModel}
                </strong>
              </div>
            </div>

            <button
              className="sidebar-back-button"
              onClick={goBack}
            >
              ← Change model
            </button>
          </div>
        </aside>

        {/* MAIN CHAT */}
        <main className="chat-main">
          {/* HEADER */}
          <header className="chat-header">
            <div className="chat-header-model">
              <div className="header-model-dot"></div>

              <div>
                <strong>
                  {selectedModelInfo?.name ??
                    selectedModel}
                </strong>

                <span>
                  {selectedRuntimeInfo?.name}
                </span>
              </div>
            </div>

            <button
              className="header-icon-button"
              onClick={createNewChat}
              title="New chat"
            >
              +
            </button>
          </header>

          {/* MESSAGES */}
          <div className="chat-messages">
            <div className="messages-container">
              {messages.length === 0 && (
                <div className="chat-empty-state">
                  <div className="empty-logo">
                    ◉
                  </div>

                  <h1>
                    How can I help you?
                  </h1>

                  <p>
                    Chat privately with your
                    local AI model.
                  </p>
                </div>
              )}

              {messages.map(
                (message, index) => (
                  <div
                    key={index}
                    className={`message-row ${
                      message.role
                    }`}
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

                      {message.role ===
                      "assistant" ? (
                        <MarkdownMessage
                          content={
                            message.content
                          }
                          messageIndex={index}
                        />
                      ) : (
                        <div className="message-text">
                          {message.content}
                        </div>
                      )}
                    </div>
                  </div>
                )
              )}

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
          </div>

          {/* ERROR */}
          {error && (
            <div className="chat-error">
              {error}
            </div>
          )}

          {/* COMPOSER */}
          <div className="chat-composer-wrapper">
            <div className="composer-box">
              <textarea
                ref={textareaRef}
                value={input}
                onChange={(event) =>
                  setInput(
                    event.target.value
                  )
                }
                onKeyDown={
                  handleInputKeyDown
                }
                placeholder="Message your local AI..."
                disabled={sending}
                rows={1}
              />

              <button
                className={`composer-mic ${
                  isListening ? "listening" : ""
                }`}
                title={
                  isListening
                    ? "Stop voice input"
                    : "Start voice input"
                }
                onClick={toggleVoiceInput}
                disabled={sending}
                aria-label={
                  isListening
                    ? "Stop voice input"
                    : "Start voice input"
                }
                aria-pressed={isListening}
              >
                {isListening ? "■" : "🎙"}
              </button>

              <button
                className="composer-send"
                onClick={sendMessage}
                disabled={
                  !input.trim() ||
                  sending
                }
                title="Send"
              >
                {sending ? "..." : "↑"}
              </button>
            </div>

            <div className="composer-hint">
              LocalAI Studio uses your selected
              local model.
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
          onClick={
            continueToPlayground
          }
        >
          Continue
        </button>
      </main>
    </div>
  );
}

export default App;
