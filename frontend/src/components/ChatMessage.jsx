import { tryParseSchedule } from "../utils/parseSchedule";
import { ScheduleCard } from "./ScheduleCard";

export function ChatMessage({ msg }) {
  const isUser = msg.role === "user";
  const schedule = !isUser ? tryParseSchedule(msg.text) : null;

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
          fontSize: "14px",
          whiteSpace: schedule ? "normal" : "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        <div
          style={{
            fontSize: "10px",
            color: "var(--sd-text-light)",
            marginBottom: "2px",
            padding: schedule ? "4px 8px 0" : 0,
          }}
        >
          {isUser ? "你" : "管家"}
        </div>
        {schedule ? (
          <ScheduleCard schedule={schedule} />
        ) : (
          msg.text
        )}
      </div>
    </div>
  );
}
