export function ProgressIndicator({ progress, done }) {
  if (!progress || progress.length === 0) return null;

  return (
    <div
      style={{
        padding: "4px 10px",
        color: "var(--sd-text-light)",
        display: "flex",
        flexDirection: "column",
        gap: "2px",
        marginBottom: "8px",
      }}
    >
      {progress.map((item, i) => {
        const isLast = i === progress.length - 1;
        const label = item.type === "error" ? "[错]" : item.type === "thinking" ? "[想]" : "[步]";
        return (
          <div key={i} style={{ display: "flex", gap: "6px", alignItems: "flex-start" }}>
            <span style={{ flexShrink: 0 }}>{label}</span>
            <span
              className={isLast && !done ? "sd-blink" : ""}
              style={{ flex: 1, whiteSpace: "pre-wrap", wordBreak: "break-word" }}
            >
              {item.text}
            </span>
          </div>
        );
      })}
    </div>
  );
}
