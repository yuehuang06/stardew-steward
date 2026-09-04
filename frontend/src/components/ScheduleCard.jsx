const ACTION_EMOJI = {
  harvest: "\u{1F33E}",
  shop: "\u{1F6D2}",
  gift: "\u{1F381}",
  water: "\u{1F4A7}",
  mine: "\u26CF\uFE0F",
  fish: "\u{1F3A3}",
  other: "\u{1F4CB}",
};

const PRIORITY_LABEL = {
  must: "\u5FC5\u505A",
  should: "\u5EFA\u8BAE",
  could: "\u53EF\u9009",
};

export function ScheduleCard({ schedule }) {
  const net = schedule.total_income - schedule.total_cost;

  return (
    <div className="sd-schedule sd-slide-in">
      <div className="sd-schedule-header">
        {"\u{1F4CB}"} {schedule.summary}
      </div>

      {schedule.tasks.map((task, i) => {
        const emoji = ACTION_EMOJI[task.action] || ACTION_EMOJI.other;
        const pri = task.priority || "other";
        return (
          <div key={i} className="sd-task-row">
            <span className="sd-task-icon">{emoji}</span>
            <span className="sd-task-desc">{task.description}</span>
            <span className="sd-task-badge sd-badge-time">
              {task.time_cost}h
            </span>
            {task.cost > 0 && (
              <span className="sd-task-badge sd-badge-cost">
                -{task.cost}g
              </span>
            )}
            {task.income > 0 && (
              <span className="sd-task-badge sd-badge-income">
                +{task.income}g
              </span>
            )}
            <span className={`sd-task-badge sd-priority-${pri}`}>
              {PRIORITY_LABEL[pri] || pri}
            </span>
          </div>
        );
      })}

      <div className="sd-schedule-footer">
        <span>{"\u23F1"} {schedule.total_time}h</span>
        <span style={{ color: "var(--sd-red)" }}>
          {"\u2212"}{schedule.total_cost}g
        </span>
        <span style={{ color: "var(--sd-green-d)" }}>
          +{schedule.total_income}g
        </span>
        <span style={{ fontWeight: 700, color: net >= 0 ? "var(--sd-green-d)" : "var(--sd-red)" }}>
          {"\u51C0"} {net >= 0 ? "+" : ""}{net}g
        </span>
      </div>

      {schedule.notes && schedule.notes.length > 0 && (
        <div className="sd-schedule-notes">
          {schedule.notes.map((n, i) => (
            <div key={i}>{"\u26A0"} {n}</div>
          ))}
        </div>
      )}
    </div>
  );
}
