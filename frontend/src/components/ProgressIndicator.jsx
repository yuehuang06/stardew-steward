export function ProgressIndicator({ progress, maxSteps, currentStep }) {
  if ((!progress || progress.length === 0) && !currentStep) return null;

  const steps = Array.from({ length: maxSteps || 10 }, (_, i) => i < (currentStep || 0));

  return (
    <div
      style={{
        padding: "2px 10px",
        color: "var(--sd-text-light)",
        marginBottom: "6px",
        display: "flex",
        flexDirection: "column",
        gap: "3px",
      }}
    >
      {progress && progress.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: "1px" }}>
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
      )}
      {(currentStep || 0) > 0 && (
        <div className="sd-progress-bar">
          {steps.map((filled, i) => (
            <div
              key={i}
              className={filled ? "sd-progress-cell filled" : "sd-progress-cell"}
            />
          ))}
        </div>
      )}
    </div>
  );
}
