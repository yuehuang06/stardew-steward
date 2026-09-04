export function ProgressIndicator({ progress }) {
  if (!progress || progress.length === 0) return null;

  return (
    <div
      style={{
        padding: "4px 10px",
        color: "var(--sd-text-light)",
        fontSize: "13px",
        display: "flex",
        flexDirection: "column",
        gap: "2px",
      }}
    >
      {progress.map((item, i) => {
        const isLast = i === progress.length - 1;
        return (
          <div key={i} style={{ display: "flex", gap: "6px", alignItems: "center" }}>
            <span>
              {item.type === "error" ? "[错] " : item.type === "thinking" ? "[想] " : "[步] "}
            </span>
            <span className={isLast ? "sd-blink" : ""} style={{ flex: 1 }}>
              {item.text}
            </span>
          </div>
        );
      })}
    </div>
  );
}
