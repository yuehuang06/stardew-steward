import { useState } from "react";

export function InputBar({ onSend, onInterrupt, loading, usageBrief }) {
  const [text, setText] = useState("");

  const handleSend = () => {
    if (!text.trim() || loading) return;
    onSend(text);
    setText("");
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
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
          className="sd-input"
          style={{ flex: 1, fontSize: "14px" }}
          placeholder="问点什麽..."
          value={text}
          onChange={(e) => setText(e.target.value)}
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
