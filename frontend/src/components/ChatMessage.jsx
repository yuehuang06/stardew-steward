import { tryParseSchedule } from "../utils/parseSchedule";
import { ScheduleCard } from "./ScheduleCard";
import { Markdown } from "./Markdown";

export function ChatMessage({ msg }) {
  const isUser = msg.role === "user";
  const typing = msg.typing;
  const schedule = !isUser && !typing ? tryParseSchedule(msg.text) : null;

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
          maxWidth: "90%",
          padding: schedule ? 0 : "6px 10px",
          background: isUser ? "var(--sd-cream)" : "var(--sd-parchment)",
          whiteSpace: schedule ? "normal" : "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        {!isUser ? null : (
          <div
            style={{
              color: "var(--sd-text-light)",
              marginBottom: "2px",
            }}
          >
            你
          </div>
        )}
        {schedule ? (
          <ScheduleCard schedule={schedule} />
        ) : isUser ? (
          msg.text
        ) : typing ? (
          <span>{msg.text}<span className="sd-blink">_</span></span>
        ) : (
          <Markdown>{msg.text}</Markdown>
        )}
      </div>
    </div>
  );
}
