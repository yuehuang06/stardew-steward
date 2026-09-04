import { getCurrentWindow } from "@tauri-apps/api/window";

export function Titlebar({ status, expanded, onToggle, onRefresh, onTop, onToggleTop, onOpenSessions }) {
  const btnStyle = { flex: 1, padding: "2px 4px", lineHeight: 1, height: "22px" };
  return (
    <>
      <div
        data-tauri-drag-region
        style={{
          flexShrink: 0,
          display: "flex",
          flexDirection: "column",
          gap: "3px",
          padding: "3px 6px",
          position: "relative",
        }}
      >
        <button
          className="sd-btn"
          style={{
            position: "absolute",
            top: "2px",
            right: "4px",
            width: "18px",
            height: "18px",
            padding: 0,
            lineHeight: 1,
            background: "var(--sd-red)",
            fontSize: "11px",
            border: "1px solid var(--sd-wood-dark)",
            boxShadow: "none",
          }}
          onClick={() => getCurrentWindow().close()}
        >
          X
        </button>
        <div data-tauri-drag-region style={{ cursor: "default", textAlign: "center" }}>
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
        <div style={{ display: "flex", gap: "3px", paddingRight: "22px" }}>
          <button className="sd-btn" style={btnStyle} onClick={onToggleTop}>
            {onTop ? "置顶" : "置底"}
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onOpenSessions}>
            历史
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onRefresh}>
            刷新
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onToggle}>
            {expanded ? "收起" : "展开"}
          </button>
        </div>
      </div>
      <div className="sd-sep" />
    </>
  );
}
