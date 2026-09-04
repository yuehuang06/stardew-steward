const ACTION_LABEL = {
  harvest: "收",
  shop: "买",
  gift: "礼",
  water: "浇",
  mine: "矿",
  fish: "钓",
  other: "·",
};

const PRIORITY_LABEL = {
  must: "必做",
  should: "建议",
  could: "可选",
};

function getPriorityInfo(pri) {
  const key = (pri || "").toLowerCase();
  const label = PRIORITY_LABEL[key] || pri || "";
  return { key, label };
}

export function ScheduleCard({ schedule }) {
  const net = schedule.total_income - schedule.total_cost;

  return (
    <div className="sd-schedule sd-slide-in">
      <div className="sd-schedule-header">
        {schedule.summary}
      </div>

      {schedule.tasks.map((task, i) => {
        const label = ACTION_LABEL[task.action] || ACTION_LABEL.other;
        const { key: priKey, label: priLabel } = getPriorityInfo(task.priority);
        return (
          <div key={i} className="sd-task-row">
            <span className="sd-task-icon">{label}</span>
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
            <span className={`sd-task-badge sd-priority-${priKey}`}>
              {priLabel}
            </span>
          </div>
        );
      })}

      <div className="sd-schedule-footer">
        <span>耗时 {schedule.total_time}h</span>
        <span style={{ color: "var(--sd-red)" }}>
          -{schedule.total_cost}g
        </span>
        <span style={{ color: "var(--sd-green-d)" }}>
          +{schedule.total_income}g
        </span>
        <span style={{ fontWeight: 700, color: net >= 0 ? "var(--sd-green-d)" : "var(--sd-red)" }}>
          净 {net >= 0 ? "+" : ""}{net}g
        </span>
      </div>

      {schedule.notes && schedule.notes.length > 0 && (
        <div className="sd-schedule-notes">
          {schedule.notes.map((n, i) => (
            <div key={i}>! {n}</div>
          ))}
        </div>
      )}
    </div>
  );
}
