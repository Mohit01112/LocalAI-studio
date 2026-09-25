import {
  useEffect,
  useRef,
  useState,
  type ChangeEvent,
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

interface Attachment {
  name: string;
  content: string;
  size: number;
}

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  attachment?: Attachment;
}

interface RecentChat {
  id: number;
  title: string;
  messages: ChatMessage[];
}

interface SpeechRecognitionResultItem {
  transcript: string;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  [index: number]: SpeechRecognitionResultItem;
}

interface SpeechRecognitionEvent {
  resultIndex: number;
  results: {
    [index: number]: SpeechRecognitionResult;
    length: number;
  };
}

interface SpeechRecognitionInstance {
  continuous: boolean;
  interimResults: boolean;
  lang: string;

  start: () => void;
  stop: () => void;

  onresult:
    | ((event: SpeechRecognitionEvent) => void)
    | null;

  onerror:
    | ((event: { error: string }) => void)
    | null;

  onend: (() => void) | null;
}

interface SpeechRecognitionConstructor {
  new (): SpeechRecognitionInstance;
}

declare global {
  interface Window {
    SpeechRecognition?: SpeechRecognitionConstructor;
    webkitSpeechRecognition?: SpeechRecognitionConstructor;
  }
}

function App() {
  const [runtimes, setRuntimes] = useState<RuntimeInfo[]>([]);
  const [models, setModels] = useState<ModelInfo[]>([]);

  const [selectedRuntime, setSelectedRuntime] =
    useState("");

  const [selectedModel, setSelectedModel] =
    useState("");

  const [loadingRuntimes, setLoadingRuntimes] =
    useState(true);

  const [loadingModels, setLoadingModels] =
    useState(false);

  const [error, setError] = useState("");

  const [showPlayground, setShowPlayground] =
    useState(false);

  const [messages, setMessages] =
    useState<ChatMessage[]>([]);

  const [input, setInput] = useState("");

  const [sending, setSending] =
    useState(false);

  const [recentChats, setRecentChats] =
    useState<RecentChat[]>([]);

  const [activeChatId, setActiveChatId] =
    useState<number | null>(null);

  const [copiedCode, setCopiedCode] =
    useState<number | null>(null);

  const [attachment, setAttachment] =
    useState<Attachment | null>(null);

  const [isListening, setIsListening] =
    useState(false);

  const messagesEndRef =
    useRef<HTMLDivElement>(null);

  const textareaRef =
    useRef<HTMLTextAreaElement>(null);

  const fileInputRef =
    useRef<HTMLInputElement>(null);

  const recognitionRef =
    useRef<SpeechRecognitionInstance | null>(null);

  useEffect(() => {
    discoverRuntimes();
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({
      behavior: "smooth",
    });
  }, [messages, sending]);

  useEffect(() => {
    return () => {
      recognitionRef.current?.stop();
    };
  }, []);

  async function discoverRuntimes() {
    try {
      setLoadingRuntimes(true);
      setError("");

      const result = await invoke<RuntimeInfo[]>(
        "discover_runtimes"
      );

      setRuntimes(result);
    } catch (err) {
      console.error(
        "Runtime discovery error:",
        err
      );

      setError(
        "Failed to detect AI runtimes."
      );
    } finally {
      setLoadingRuntimes(false);
    }
  }

  async function selectRuntime(
    runtimeId: string
  ) {
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

      const result =
        await invoke<ModelInfo[]>(
          "provider_get_models",
          {
            runtimeId,
          }
        );

      setModels(result);
    } catch (err) {
      console.error(
        "Model detection error:",
        err
      );

      const runtime = runtimes.find(
        (item) => item.id === runtimeId
      );

      setError(
        `Could not connect to ${
          runtime?.name ??
          "the selected runtime"
        }.`
      );
    } finally {
      setLoadingModels(false);
    }
  }

  function continueToPlayground() {
    if (
      !selectedRuntime ||
      !selectedModel
    ) {
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
    setAttachment(null);
    setError("");
  }

  function createNewChat() {
    setMessages([]);
    setInput("");
    setAttachment(null);
    setError("");
    setActiveChatId(null);

    setTimeout(() => {
      textareaRef.current?.focus();
    }, 50);
  }

  function openRecentChat(
    chat: RecentChat
  ) {
    setActiveChatId(chat.id);
    setMessages(chat.messages);
    setInput("");
    setAttachment(null);
    setError("");
  }

  function saveCurrentChat(
    updatedMessages: ChatMessage[]
  ) {
    if (updatedMessages.length === 0) {
      return;
    }

    const firstUserMessage =
      updatedMessages.find(
        (message) =>
          message.role === "user"
      );

    if (!firstUserMessage) {
      return;
    }

    const title =
      firstUserMessage.content.length > 40
        ? `${firstUserMessage.content.slice(
            0,
            40
          )}...`
        : firstUserMessage.content;

    setRecentChats((previous) => {
      if (activeChatId !== null) {
        return previous.map((chat) =>
          chat.id === activeChatId
            ? {
                ...chat,
                title,
                messages:
                  updatedMessages,
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

  async function handleFileSelect(
    event: ChangeEvent<HTMLInputElement>
  ) {
    const file =
      event.target.files?.[0];

    if (!file) {
      return;
    }

    setError("");

    /*
     * Keep the first implementation focused
     * on text/code/data files that local LLMs
     * can directly understand.
     */
    const allowedExtensions = [
      ".txt",
      ".md",
      ".py",
      ".js",
      ".jsx",
      ".ts",
      ".tsx",
      ".json",
      ".csv",
      ".html",
      ".css",
      ".xml",
      ".yaml",
      ".yml",
      ".sql",
      ".java",
      ".c",
      ".cpp",
      ".h",
      ".hpp",
      ".rs",
      ".go",
      ".php",
      ".rb",
      ".sh",
      ".bat",
      ".log",
    ];

    const lowerName =
      file.name.toLowerCase();

    const isTextFile =
      file.type.startsWith("text/") ||
      allowedExtensions.some(
        (extension) =>
          lowerName.endsWith(extension)
      );

    if (!isTextFile) {
      setError(
        "For now, LocalAI Studio supports text, code and data files for chat attachments."
      );

      event.target.value = "";
      return;
    }

    if (file.size > 5 * 1024 * 1024) {
      setError(
        "Please select a file smaller than 5 MB."
      );

      event.target.value = "";
      return;
    }

    try {
      const content =
        await file.text();

      setAttachment({
        name: file.name,
        content,
        size: file.size,
      });
    } catch (err) {
      console.error(
        "File reading error:",
        err
      );

      setError(
        "Could not read the selected file."
      );
    }

    event.target.value = "";
  }

  function removeAttachment() {
    setAttachment(null);

    setTimeout(() => {
      textareaRef.current?.focus();
    }, 50);
  }

  function openFilePicker() {
    fileInputRef.current?.click();
  }

  function startListening() {
    const SpeechRecognition =
      window.SpeechRecognition ||
      window.webkitSpeechRecognition;

    if (!SpeechRecognition) {
      setError(
        "Speech recognition is not available in this desktop WebView."
      );

      return;
    }

    if (isListening) {
      recognitionRef.current?.stop();
      setIsListening(false);
      return;
    }

    setError("");

    const recognition =
      new SpeechRecognition();

    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = "en-US";

    recognition.onresult = (
      event: SpeechRecognitionEvent
    ) => {
      let transcript = "";

      for (
        let i = event.resultIndex;
        i < event.results.length;
        i++
      ) {
        transcript +=
          event.results[i][0]
            .transcript;
      }

      if (transcript.trim()) {
        setInput((previous) => {
          const separator =
            previous.trim().length > 0
              ? " "
              : "";

          return (
            previous +
            separator +
            transcript.trim()
          );
        });
      }
    };

    recognition.onerror = (
      event
    ) => {
      console.error(
        "Speech recognition error:",
        event.error
      );

      setIsListening(false);

      if (
        event.error ===
        "not-allowed"
      ) {
        setError(
          "Microphone permission was denied."
        );
      } else {
        setError(
          "Speech recognition stopped. Please try again."
        );
      }
    };

    recognition.onend = () => {
      setIsListening(false);
    };

    recognitionRef.current =
      recognition;

    try {
      recognition.start();
      setIsListening(true);
    } catch (err) {
      console.error(
        "Could not start microphone:",
        err
      );

      setIsListening(false);

      setError(
        "Could not start microphone input."
      );
    }
  }

  async function sendMessage() {
    const message =
      input.trim();

    if (
      !message &&
      !attachment
    ) {
      return;
    }

    if (
      !selectedRuntime ||
      !selectedModel ||
      sending
    ) {
      return;
    }

    if (isListening) {
      recognitionRef.current?.stop();
      setIsListening(false);
    }

    const userMessage:
      ChatMessage = {
      role: "user",
      content: message,
      attachment:
        attachment ?? undefined,
    };

    const conversation = [
      ...messages,
      userMessage,
    ];

    /*
     * The UI keeps the message clean,
     * while the local model receives
     * the attached file content.
     */
    const modelConversation =
      conversation.map(
        (chatMessage) => {
          if (
            chatMessage.attachment
          ) {
            return {
              role: chatMessage.role,
              content:
                `${chatMessage.content}\n\n` +
                `--- Attached file: ${chatMessage.attachment.name} ---\n\n` +
                chatMessage.attachment.content +
                `\n\n--- End attached file ---`,
            };
          }

          return {
            role: chatMessage.role,
            content:
              chatMessage.content,
          };
        }
      );

    setMessages(conversation);
    setInput("");
    setAttachment(null);
    setError("");
    setSending(true);

    try {
      const response =
        await invoke<{
          message: {
            role: string;
            content: string;
          };
          done: boolean;
          provider?: string;
          model?: string;
        }>("provider_chat", {
          runtimeId:
            selectedRuntime,

          request: {
            model: selectedModel,

            messages:
              modelConversation,

            stream: false,
          },
        });

      const assistantMessage:
        ChatMessage = {
        role: "assistant",
        content:
          response.message.content,
      };

      const updatedMessages = [
        ...conversation,
        assistantMessage,
      ];

      setMessages(
        updatedMessages
      );

      saveCurrentChat(
        updatedMessages
      );
    } catch (err) {
      console.error(
        "Chat error:",
        err
      );

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
      await navigator.clipboard.writeText(
        code
      );

      setCopiedCode(index);

      setTimeout(() => {
        setCopiedCode(null);
      }, 2000);
    } catch (err) {
      console.error(
        "Copy failed:",
        err
      );
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

              const code =
                String(children).replace(
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

  const selectedRuntimeInfo =
    runtimes.find(
      (runtime) =>
        runtime.id ===
        selectedRuntime
    );

  const selectedModelInfo =
    models.find(
      (model) =>
        model.name ===
        selectedModel
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
                recentChats.map(
                  (chat) => (
                    <button
                      key={chat.id}
                      className={`recent-chat ${
                        activeChatId ===
                        chat.id
                          ? "active"
                          : ""
                      }`}
                      onClick={() =>
                        openRecentChat(
                          chat
                        )
                      }
                    >
                      <span className="recent-chat-icon">
                        ◌
                      </span>

                      <span className="recent-chat-title">
                        {chat.title}
                      </span>
                    </button>
                  )
                )
              )}

            </div>

          </div>

          <div className="sidebar-bottom">

            <div className="sidebar-model">

              <span className="sidebar-status-dot"></span>

              <div>

                <small>
                  {
                    selectedRuntimeInfo?.name
                  }
                </small>

                <strong>
                  {
                    selectedModelInfo?.name ??
                    selectedModel
                  }
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
                  {
                    selectedModelInfo?.name ??
                    selectedModel
                  }
                </strong>

                <span>
                  {
                    selectedRuntimeInfo?.name
                  }
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
                    Chat privately with
                    your local AI model.
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

                    <div
                      className={`message-avatar ${
                        message.role
                      }-avatar`}
                    >
                      {message.role ===
                      "user"
                        ? "U"
                        : "AI"}
                    </div>

                    <div className="message-content">

                      <div className="message-role">

                        {message.role ===
                        "user"
                          ? "You"
                          : "Local AI"}

                      </div>

                      {message.attachment && (
                        <div className="message-attachment">

                          <span className="attachment-icon">
                            📎
                          </span>

                          <div>
                            <strong>
                              {
                                message
                                  .attachment
                                  .name
                              }
                            </strong>

                            <small>
                              Attached file
                            </small>
                          </div>

                        </div>
                      )}

                      {message.role ===
                      "assistant" ? (
                        <MarkdownMessage
                          content={
                            message.content
                          }
                          messageIndex={
                            index
                          }
                        />
                      ) : (
                        <div className="message-text">
                          {
                            message.content
                          }
                        </div>
                      )}

                    </div>

                  </div>
                )
              )}

              {sending && (
                <div className="message-row assistant">

                  <div className="message-avatar assistant-avatar">
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

              <div
                ref={messagesEndRef}
              />

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

            {attachment && (
              <div className="attachment-preview">

                <div className="attachment-preview-icon">
                  📎
                </div>

                <div className="attachment-preview-info">

                  <strong>
                    {attachment.name}
                  </strong>

                  <span>
                    {(
                      attachment.size /
                      1024
                    ).toFixed(1)}{" "}
                    KB
                  </span>

                </div>

                <button
                  className="attachment-remove"
                  onClick={
                    removeAttachment
                  }
                  title="Remove file"
                >
                  ×
                </button>

              </div>
            )}

            <div className="composer-box">

              {/* FILE UPLOAD */}

              <button
                className="composer-plus"
                onClick={
                  openFilePicker
                }
                title="Attach file"
              >
                +
              </button>

              <input
                ref={fileInputRef}
                type="file"
                className="hidden-file-input"
                onChange={
                  handleFileSelect
                }
              />

              {/* INPUT */}

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

              {/* MICROPHONE */}

              <button
                className={`composer-mic ${
                  isListening
                    ? "mic-active"
                    : ""
                }`}
                title={
                  isListening
                    ? "Stop listening"
                    : "Voice input"
                }
                onClick={
                  startListening
                }
              >
                {isListening
                  ? "●"
                  : "🎙"}
              </button>

              {/* SEND */}

              <button
                className="composer-send"
                onClick={
                  sendMessage
                }
                disabled={
                  (!input.trim() &&
                    !attachment) ||
                  sending
                }
                title="Send"
              >
                {sending
                  ? "..."
                  : "↑"}
              </button>

            </div>

            <div className="composer-hint">
              Enter to send · Shift + Enter
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
          Discover and manage your local AI
          models.
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
              (runtime) =>
                runtime.installed
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

                  {model.size !==
                  "Unknown"
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
                {
                  selectedRuntimeInfo.name
                }
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