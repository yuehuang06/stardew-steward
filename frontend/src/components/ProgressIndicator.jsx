export function ProgressIndicator({ progress, done }) {
  if (!progress || progress.length === 0) return null;

  return (
    <div
      style={{
        padding: "2px 10px",
        color: "var(--sd-text-light)",
        display: "flex",
        flexDirection: "column",
        gap: "1px",
        marginBottom: "6px",
      }}
    >
      {progress.map((item, i) => {
        const isLast = i === progress.length - 1;
        return (
          <div
            key={i}
            className={isLast && !done ? "sd-blink" : ""}
            style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}
          >
            {item.text}
          </div>
        );
      })}
    </div>
  );
}
