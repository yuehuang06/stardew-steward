import { useEffect, useRef } from "react";
import { useChat } from "./hooks/useChat";
import { useWindowState, useSaveStatus, useUsage } from "./hooks/useWindowState";
import { Titlebar } from "./components/Titlebar";
import { ChatMessage } from "./components/ChatMessage";
import { ProgressIndicator } from "./components/ProgressIndicator";
import { InputBar } from "./components/InputBar";

export default function App() {
  const { messages, loading, progress, send, interrupt } = useChat();
  const { status, loading: statusLoading, refresh: refreshStatus } =
    useSaveStatus();
  const { expanded, toggle } = useWindowState();
  const { brief, refresh: refreshUsage } = useUsage();
  const scrollRef = useRef(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, progress]);

  useEffect(() => {
    if (!loading) {
      refreshUsage();
    }
  }, [loading, refreshUsage]);

  const handleSend = async (text) => {
    await send(text);
    refreshStatus();
  };

  return (
    <div
      className="sd-panel"
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        width: "100vw",
        borderRadius: "8px",
        overflow: "hidden",
      }}
    >
      <Titlebar
        status={status}
        expanded={expanded}
        onToggle={toggle}
        onRefresh={refreshStatus}
      />

      {/* 聊天区 */}
      <div
        ref={scrollRef}
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "8px",
          background: "var(--sd-parchment)",
        }}
      >
        {messages.length === 0 && !loading && (
          <div
            style={{
              textAlign: "center",
              marginTop: "40px",
              color: "var(--sd-text-light)",
              fontSize: "13px",
            }}
          >
            🌾 欢迎回来，农场主！
            <br />
            <br />
            问我「今天该干嘛」
            <br />
            或者「蓝莓什么季节种」
          </div>
        )}

        {messages.map((msg, i) => (
          <ChatMessage key={i} msg={msg} />
        ))}

        {loading && <ProgressIndicator progress={progress} />}
      </div>

      <InputBar
        onSend={handleSend}
        onInterrupt={interrupt}
        loading={loading}
        usageBrief={brief}
      />
    </div>
  );
}
