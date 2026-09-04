import { useEffect, useRef, useState } from "react";
import { useChat } from "./hooks/useChat";
import { useWindowState, useSaveStatus, useUsage } from "./hooks/useWindowState";
import { useSessions } from "./hooks/useSessions";
import { useAlwaysOnTop } from "./hooks/useAlwaysOnTop";
import { Titlebar } from "./components/Titlebar";
import { ChatMessage } from "./components/ChatMessage";
import { ProgressIndicator } from "./components/ProgressIndicator";
import { InputBar } from "./components/InputBar";
import { SessionPanel } from "./components/SessionPanel";

export default function App() {
  const { messages, loading, progress, send, interrupt, loadSession } = useChat();
  const { status, loading: statusLoading, refresh: refreshStatus } =
    useSaveStatus();
  const { expanded, toggle } = useWindowState();
  const { brief, refresh: refreshUsage } = useUsage();
  const { sessions, load, refresh: refreshSessions } = useSessions();
  const { onTop, toggle: toggleTop } = useAlwaysOnTop();
  const [showSessions, setShowSessions] = useState(false);
  const scrollRef = useRef(null);

  const hasProgress = progress.length > 0;
  const lastMsg = messages[messages.length - 1];
  const showProgressBeforeLast = hasProgress && lastMsg && lastMsg.role === "assistant" && !loading;

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
    refreshSessions();
  };

  const handleLoadSession = async (name) => {
    await loadSession(name);
    setShowSessions(false);
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
        overflow: "hidden",
        position: "relative",
      }}
    >
      <Titlebar
        status={status}
        expanded={expanded}
        onToggle={toggle}
        onRefresh={refreshStatus}
        onTop={onTop}
        onToggleTop={toggleTop}
        onOpenSessions={() => {
          refreshSessions();
          setShowSessions(true);
        }}
      />

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
            }}
          >
            欢迎回来，农场主！
            <br />
            <br />
            问我「今天该干嘛」
            <br />
            或者「蓝莓什么季节种」
          </div>
        )}

        {messages.map((msg, i) => {
          const isLast = i === messages.length - 1;
          const showProgressHere = isLast && showProgressBeforeLast;
          return (
            <div key={i}>
              {showProgressHere && (
                <ProgressIndicator progress={progress} done />
              )}
              <ChatMessage msg={msg} />
            </div>
          );
        })}

        {/* While loading (agent working, no reply yet): progress at the end */}
        {loading && hasProgress && (
          <ProgressIndicator progress={progress} done={false} />
        )}
      </div>

      <InputBar
        onSend={handleSend}
        onInterrupt={interrupt}
        loading={loading}
        usageBrief={brief}
      />

      {showSessions && (
        <SessionPanel
          sessions={sessions}
          onLoad={handleLoadSession}
          onClose={() => setShowSessions(false)}
        />
      )}
    </div>
  );
}
