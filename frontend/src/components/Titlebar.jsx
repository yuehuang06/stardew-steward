export function Titlebar({ status, expanded, onToggle, onRefresh, onTop, onToggleTop, onOpenSessions }) {
  const btnStyle = { fontSize: "12px", padding: "1px 4px", lineHeight: 1, minWidth: "24px", height: "22px", display: "flex", alignItems: "center", justifyContent: "center" };
  return (
    <div
      data-tauri-drag-region
      style={{
        display: "flex",
        alignItems: "center",
        gap: "3px",
        padding: "3px 6px",
        background: "var(--sd-wood)",
        color: "var(--sd-cream)",
        borderBottom: "2px solid var(--sd-wood-dark)",
        cursor: "default",
        flexShrink: 0,
      }}
    >
      <span style={{ fontSize: "13px", flex: 1, fontFamily: "var(--sd-font-body)" }} data-tauri-drag-region>
        {status ? (
          <>
            {status.date} · {status.money}g · {status.weather}
          </>
        ) : (
          "星露谷农场管家"
        )}
      </span>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onToggleTop}
      >
        {onTop ? "置顶" : "置底"}
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onOpenSessions}
      >
        会话
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onRefresh}
      >
        刷新
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onToggle}
      >
        {expanded ? "收" : "展"}
      </button>
    </div>
  );
}
