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
