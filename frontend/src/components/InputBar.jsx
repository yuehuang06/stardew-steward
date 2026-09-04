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
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "4px",
        padding: "6px 8px",
        background: "var(--sd-wood)",
        borderTop: "2px solid var(--sd-wood-dark)",
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
          <button className="sd-btn sd-btn-green" onClick={handleSend}>
            发送
          </button>
        )}
      </div>
      {usageBrief && (
        <div
          style={{
            fontSize: "10px",
            color: "var(--sd-parchment-d)",
            textAlign: "right",
          }}
        >
          {usageBrief}
        </div>
      )}
    </div>
  );
}
