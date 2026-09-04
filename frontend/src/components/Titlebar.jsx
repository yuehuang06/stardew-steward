export function Titlebar({ status, expanded, onToggle, onRefresh, onTop, onToggleTop, onOpenSessions }) {
  const btnStyle = { flex: 1, fontSize: "11px", padding: "2px 4px", lineHeight: 1, height: "20px" };
  return (
    <div
      style={{
        background: "var(--sd-wood)",
        color: "var(--sd-cream)",
        borderBottom: "2px solid var(--sd-wood-dark)",
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
        gap: "3px",
        padding: "3px 6px",
      }}
    >
      <div data-tauri-drag-region style={{ fontSize: "12px", textAlign: "center", cursor: "default" }}>
        {status ? (
          <>
            {status.date} · {status.money}g · {status.weather}
          </>
        ) : (
          "星露谷农场管家"
        )}
      </div>
      <div style={{ display: "flex", gap: "3px" }}>
        <button className="sd-btn" style={btnStyle} onClick={onToggleTop}>
          {onTop ? "置顶" : "置底"}
        </button>
        <button className="sd-btn" style={btnStyle} onClick={onOpenSessions}>
          会话
        </button>
        <button className="sd-btn" style={btnStyle} onClick={onRefresh}>
          刷新
        </button>
        <button className="sd-btn" style={btnStyle} onClick={onToggle}>
          {expanded ? "收起" : "展开"}
        </button>
      </div>
    </div>
  );
}
