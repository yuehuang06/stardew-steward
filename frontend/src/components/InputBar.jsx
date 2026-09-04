import { useRef } from "react";

export function InputBar({ onSend, onInterrupt, loading, usageBrief }) {
  const inputRef = useRef(null);

  const handleSend = () => {
    const text = inputRef.current?.value ?? "";
    if (!text.trim() || loading) return;
    onSend(text);
    if (inputRef.current) {
      inputRef.current.value = "";
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <>
      <div className="sd-sep" />
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "4px",
            padding: "10px 8px 6px",
            flexShrink: 0,
          }}
        >
        <div style={{ display: "flex", gap: "4px" }}>
          <input
            ref={inputRef}
            type="text"
            lang="zh-CN"
            className="sd-input"
            style={{ flex: 1 }}
            placeholder="问点什么..."
            onKeyDown={handleKeyDown}
            disabled={loading}
          />
          {loading ? (
            <button
              className="sd-btn"
              style={{ background: "var(--sd-red)" }}
              onClick={onInterrupt}
            >
              停止
            </button>
          ) : (
            <button
              className="sd-btn sd-btn-green"
              style={{ width: "24px", height: "24px", padding: 0, display: "flex", alignItems: "center", justifyContent: "center" }}
              onClick={handleSend}
            >
              {"\u2192"}
            </button>
          )}
        </div>
        {usageBrief && (
          <div
            style={{
              fontSize: "10px",
              color: "var(--sd-text)",
              textAlign: "right",
            }}
          >
            {usageBrief}
          </div>
        )}
      </div>
    </>
  );
}
