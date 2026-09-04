export function ProgressIndicator({ progress }) {
  if (!progress) return null;

  return (
    <div
      style={{
        padding: "2px 10px",
        color: "var(--sd-text-light)",
        marginBottom: "6px",
      }}
    >
      <span className="sd-blink" style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
        {progress.text}
      </span>
    </div>
  );
}
