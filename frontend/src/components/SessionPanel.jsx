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

export function SessionPanel({ sessions, onLoad, onDelete, onClose }) {
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
        <span style={{ flex: 1 }}>历史会话</span>
        <button
          className="sd-btn"
          style={{ padding: "2px 6px" }}
          onClick={onClose}
        >
          X
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "4px 8px" }}>
        {sessions.length === 0 ? (
          <div
            style={{
              textAlign: "center",
              marginTop: "20px",
              color: "var(--sd-text-light)",
            }}
          >
            还没有历史会话
            <br />
            <br />
            对话结束后会自动保存
          </div>
        ) : (
          sessions.map((s, i) => (
            <div
              key={i}
              style={{
                padding: "6px 8px",
                marginBottom: "4px",
                background: "var(--sd-cream)",
                border: "2px solid var(--sd-wood-light)",
                cursor: "pointer",
                transition: "filter 0.1s",
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.filter = "brightness(0.95)")}
              onMouseLeave={(e) => (e.currentTarget.style.filter = "none")}
              onClick={() => onLoad(s.name)}
            >
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontWeight: 600 }}>
                  {s.name}
                </div>
                <div
                  style={{
                    display: "flex",
                    gap: "8px",
                    color: "var(--sd-text-light)",
                    marginTop: "2px",
                  }}
                >
                  <span>{formatTime(s.saved_at)}</span>
                  <span>消息 {s.message_count}</span>
                  <span>轮次 {s.interaction_rounds}</span>
                </div>
              </div>
              <button
                className="sd-btn"
                style={{
                  flexShrink: 0,
                  background: "var(--sd-red)",
                  width: "20px",
                  height: "20px",
                  padding: 0,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  border: "2px solid var(--sd-wood-darker)",
                  boxShadow: "none",
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  if (confirm(`删除会话「${s.name}」？`)) {
                    onDelete(s.name);
                  }
                }}
              >
                X
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
