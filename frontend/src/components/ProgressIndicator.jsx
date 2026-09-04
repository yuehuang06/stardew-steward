export function ProgressIndicator({ progress }) {
  if (!progress || progress.length === 0) return null;

  return (
    <div
      style={{
        padding: "2px 10px",
        color: "var(--sd-text-light)",
        marginBottom: "6px",
        display: "flex",
        flexDirection: "column",
        gap: "1px",
      }}
    >
      {progress.map((item, i) => {
        const isLast = i === progress.length - 1;
        return (
          <div
            key={i}
            className={isLast ? "sd-blink" : ""}
            style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}
          >
            {item.text}
          </div>
        );
      })}
    </div>
  );
}
