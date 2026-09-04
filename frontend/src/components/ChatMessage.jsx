export function ChatMessage({ msg }) {
  const isUser = msg.role === "user";

  return (
    <div
      className="sd-slide-in"
      style={{
        display: "flex",
        justifyContent: isUser ? "flex-end" : "flex-start",
        marginBottom: "8px",
      }}
    >
      <div
        className="sd-panel"
        style={{
          maxWidth: "85%",
          padding: "6px 10px",
          background: isUser ? "var(--sd-cream)" : "var(--sd-parchment)",
          fontSize: "14px",
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        <div
          style={{
            fontSize: "10px",
            color: "var(--sd-text-light)",
            marginBottom: "2px",
          }}
        >
          {isUser ? "你" : "管家"}
        </div>
        {msg.text}
      </div>
    </div>
  );
}
