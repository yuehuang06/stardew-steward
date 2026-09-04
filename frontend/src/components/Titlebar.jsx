export function Titlebar({ status, expanded, onToggle, onRefresh, onTop, onToggleTop, onOpenSessions }) {
  return (
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
        cursor: "move",
        flexShrink: 0,
      }}
    >
      <span style={{ fontSize: "13px", flex: 1 }} data-tauri-drag-region>
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
        style={{ fontSize: "11px", padding: "2px 6px" }}
        onClick={onToggleTop}
        title={onTop ? "取消置顶" : "始终置顶"}
      >
        {onTop ? "\u{1F4CC}" : "\u{1F5FA}"}
      </button>
      <button
        className="sd-btn"
        style={{ fontSize: "11px", padding: "2px 6px" }}
        onClick={onOpenSessions}
        title="会话管理"
      >
        {"\u{1F4C2}"}
      </button>
      <button
        className="sd-btn"
        style={{ fontSize: "11px", padding: "2px 6px" }}
        onClick={onRefresh}
        title="刷新存档"
      >
        {"\u27F3"}
      </button>
      <button
        className="sd-btn"
        style={{ fontSize: "11px", padding: "2px 6px" }}
        onClick={onToggle}
        title={expanded ? "收起" : "展开"}
      >
        {expanded ? "\u25B6" : "\u25C0"}
      </button>
    </div>
  );
}
