import { getCurrentWindow } from "@tauri-apps/api/window";

const iconBtnStyle = {
  padding: 0,
  lineHeight: 1,
  width: "24px",
  height: "24px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  fontSize: "13px",
};

export function Titlebar({ status, expanded, onToggle, onSettings, onNew, onTop, onToggleTop, onOpenSessions, onRefresh }) {
  const handleMouseDown = async (e) => {
    if (e.target.closest("button")) return;
    await getCurrentWindow().startDragging();
  };

  return (
    <>
      <div
        onMouseDown={handleMouseDown}
        style={{
          flexShrink: 0,
          display: "flex",
          flexDirection: "column",
          gap: "3px",
          padding: "3px 6px",
          position: "relative",
          cursor: "move",
        }}
      >
        <button
          className="sd-btn"
          style={{
            position: "absolute",
            top: "2px",
            left: "4px",
            ...iconBtnStyle,
            background: "var(--sd-green)",
            border: "3px solid var(--sd-wood-darker)",
            boxShadow: "none",
          }}
          title="刷新"
          onClick={onRefresh}
        >
          ↻
        </button>
        <button
          className="sd-btn"
          style={{
            position: "absolute",
            top: "2px",
            right: "4px",
            ...iconBtnStyle,
            background: "var(--sd-red)",
            border: "3px solid var(--sd-wood-darker)",
            boxShadow: "none",
          }}
          title="关闭"
          onClick={() => getCurrentWindow().close()}
        >
          ✕
        </button>
        <div style={{ textAlign: "center" }}>
          {status ? (
            <>
              <div>
                {status.date} · {status.money}g
              </div>
              <div>
                {status.weather}
              </div>
            </>
          ) : (
            <div>星露谷农场管家</div>
          )}
        </div>
        <div style={{ display: "flex", justifyContent: "space-around", paddingLeft: "28px", paddingRight: "28px" }}>
          <button className="sd-btn" style={iconBtnStyle} title="新建会话" onClick={onNew}>
            ＋
          </button>
          <button className="sd-btn" style={iconBtnStyle} title={onTop ? "取消置顶" : "始终置顶"} onClick={onToggleTop}>
            {onTop ? "📌" : "↑"}
          </button>
          <button className="sd-btn" style={iconBtnStyle} title="历史会话" onClick={onOpenSessions}>
            🕐
          </button>
          <button className="sd-btn" style={iconBtnStyle} title="设置" onClick={onSettings}>
            ⚙
          </button>
          <button className="sd-btn" style={iconBtnStyle} title={expanded ? "收起" : "展开"} onClick={onToggle}>
            {expanded ? "🗗" : "🗖"}
          </button>
        </div>
      </div>
      <div className="sd-sep" />
    </>
  );
}
