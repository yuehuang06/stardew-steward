import { getCurrentWindow } from "@tauri-apps/api/window";

export function Titlebar({ status, expanded, onToggle, onSettings, onNew, onTop, onToggleTop, onOpenSessions }) {
  const btnStyle = { padding: "2px 4px", lineHeight: 1, height: "22px" };
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
            width: "24px",
            height: "24px",
            padding: 0,
            lineHeight: 1,
            background: "var(--sd-red)",
            border: "3px solid var(--sd-wood-darker)",
            boxShadow: "none",
          }}
          onClick={() => getCurrentWindow().close()}
        >
          X
        </button>
        <div data-tauri-drag-region style={{ textAlign: "center" }}>
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
        <div data-tauri-drag-region style={{ display: "flex", justifyContent: "space-around", paddingRight: "28px" }}>
          <button className="sd-btn" style={btnStyle} onClick={onNew}>
            新建
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onToggleTop}>
            {onTop ? "置顶" : "置底"}
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onOpenSessions}>
            历史
          </button>
          <button className="sd-btn" style={btnStyle} onClick={onSettings}>
            设置
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
