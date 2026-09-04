import { useState } from "react";

function formatTime(ts) {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const now = new Date();
  const diff = (now - d) / 1000;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return Math.floor(diff / 60) + "分钟前";
  if (diff < 86400) return Math.floor(diff / 3600) + "小时前";
  return `${d.getMonth() + 1}/${d.getDate()} ${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
}

export function SessionPanel({ sessions, onSave, onLoad, onClose }) {
  const [name, setName] = useState("");
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    const n = name.trim() || "untitled";
    setSaving(true);
    try {
      await onSave(n);
      setName("");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        bottom: 0,
        width: "100%",
        background: "var(--sd-parchment)",
        borderLeft: "3px solid var(--sd-wood)",
        display: "flex",
        flexDirection: "column",
        zIndex: 10,
      }}
      className="sd-slide-in"
    >
      <div
        data-tauri-drag-region
        style={{
          display: "flex",
          alignItems: "center",
          gap: "6px",
          padding: "4px 8px",
          background: "var(--sd-wood)",
          color: "var(--sd-cream)",
          borderBottom: "2px solid var(--sd-wood-dark)",
          flexShrink: 0,
        }}
      >
        <span style={{ flex: 1, fontSize: "13px" }}>{"\u{1F4C2}"} 会话管理</span>
        <button
          className="sd-btn"
          style={{ fontSize: "11px", padding: "2px 6px" }}
          onClick={onClose}
          title="关闭"
        >
          {"\u2715"}
        </button>
      </div>

      <div style={{ padding: "6px 8px", flexShrink: 0, display: "flex", gap: "4px" }}>
        <input
          className="sd-input"
          style={{ flex: 1, fontSize: "13px" }}
          placeholder="会话名称..."
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSave();
          }}
          disabled={saving}
        />
        <button
          className="sd-btn sd-btn-green"
          style={{ fontSize: "12px" }}
          onClick={handleSave}
          disabled={saving}
        >
          保存
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "4px 8px" }}>
        {sessions.length === 0 ? (
          <div
            style={{
              textAlign: "center",
              marginTop: "20px",
              color: "var(--sd-text-light)",
              fontSize: "12px",
            }}
          >
            {"\u{1F4C4}"} 还没有保存的会话
          </div>
        ) : (
          sessions.map((s, i) => (
            <div
              key={i}
              onClick={() => onLoad(s.name)}
              style={{
                padding: "6px 8px",
                marginBottom: "4px",
                background: "var(--sd-cream)",
                border: "2px solid var(--sd-wood-light)",
                cursor: "pointer",
                transition: "filter 0.1s",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.filter = "brightness(0.95)")}
              onMouseLeave={(e) => (e.currentTarget.style.filter = "none")}
            >
              <div style={{ fontSize: "13px", fontWeight: 600 }}>
                {s.name}
              </div>
              <div
                style={{
                  display: "flex",
                  gap: "8px",
                  fontSize: "10px",
                  color: "var(--sd-text-light)",
                  marginTop: "2px",
                }}
              >
                <span>{formatTime(s.saved_at)}</span>
                <span>{"\u{1F4AC}"} {s.message_count}</span>
                <span>{"\u{1F501}"} {s.interaction_rounds}</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
