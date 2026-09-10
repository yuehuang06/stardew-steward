import { tryParseSchedule } from "../utils/parseSchedule";
import { ScheduleCard } from "./ScheduleCard";
import { Markdown } from "./Markdown";

export function ChatMessage({ msg }) {
  const isUser = msg.role === "user";
  const typing = msg.typing;
  const schedule = !isUser && !typing ? tryParseSchedule(msg.text) : null;
  const trace = !isUser && !typing && msg.trace && msg.trace.length > 0 ? msg.trace : null;

  return (
    <div
      className="sd-slide-in"
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: isUser ? "flex-end" : "flex-start",
        marginBottom: "8px",
        opacity: msg.queued ? 0.55 : 1,
      }}
    >
      {trace && (
        <div
          style={{
            fontSize: "10px",
            lineHeight: 1.6,
            color: "var(--sd-text-light)",
            padding: "0 10px",
            marginBottom: "2px",
            opacity: 0.9,
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
          }}
        >
          {trace.map((t, i) => (
            <div key={i}>
              {t.type === "thinking" ? "💭 " : t.type === "error" ? "✗ " : "· "}
              {t.text}
            </div>
          ))}
        </div>
      )}
      <div
        className="sd-bubble"
        style={{
          maxWidth: "90%",
          padding: schedule ? 0 : "4px 8px",
          whiteSpace: schedule ? "normal" : "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        {schedule ? (
          <ScheduleCard schedule={schedule} />
        ) : isUser ? (
          msg.queued ? `${msg.text}（排队中…）` : msg.text
        ) : typing ? (
          <span>{msg.text}<span className="sd-blink">_</span></span>
        ) : (
          <Markdown>{msg.text}</Markdown>
        )}
      </div>
    </div>
  );
}
