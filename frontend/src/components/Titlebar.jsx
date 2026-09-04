export function Titlebar({ status, expanded, onToggle, onRefresh, onTop, onToggleTop, onOpenSessions }) {
  const btnStyle = { fontSize: "12px", padding: "1px 5px", lineHeight: 1, width: "22px", height: "22px", display: "flex", alignItems: "center", justifyContent: "center" };
  return (
    <div
      data-tauri-drag-region
      style={{
        display: "flex",
        alignItems: "center",
        gap: "4px",
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
        title={onTop ? "取消置顶" : "始终置顶"}
      >
        {onTop ? "\u{1F4CC}" : "\u{1F5FA}"}
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onOpenSessions}
        title="会话管理"
      >
        {"\u{1F4C2}"}
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onRefresh}
        title="刷新存档"
      >
        {"\u27F3"}
      </button>
      <button
        className="sd-btn"
        style={btnStyle}
        onClick={onToggle}
        title={expanded ? "收起" : "展开"}
      >
        {expanded ? "\u25B6" : "\u25C0"}
      </button>
    </div>
  );
}
