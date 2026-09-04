export function ProgressIndicator({ progress }) {
  if (!progress || progress.length === 0) return null;

  const last = progress[progress.length - 1];

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "6px",
        padding: "4px 10px",
        color: "var(--sd-text-light)",
        fontSize: "12px",
      }}
    >
      <span className="sd-blink">▸</span>
      <span>
        {last.type === "error" ? "✗ " : last.type === "thinking" ? "💭 " : "→ "}
        {last.text}
      </span>
    </div>
  );
}
