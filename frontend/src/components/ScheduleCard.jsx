const ACTION_META = {
  harvest: { icon: "🌾", label: "收获" },
  shop:    { icon: "🛒", label: "购物" },
  gift:    { icon: "🎁", label: "送礼" },
  water:   { icon: "💧", label: "浇水" },
  mine:    { icon: "⛏", label: "下矿" },
  fish:    { icon: "🎣", label: "钓鱼" },
  other:   { icon: "📋", label: "其他" },
};

const PRIORITY_META = {
  must:   { label: "必做", cls: "must" },
  should: { label: "建议", cls: "should" },
  could:  { label: "可选", cls: "could" },
};

function getActionMeta(action) {
  const key = (action || "other").toLowerCase();
  return ACTION_META[key] || ACTION_META.other;
}

function getPriorityMeta(priority) {
  const key = (priority || "").toLowerCase();
  return PRIORITY_META[key] || { label: priority || "", cls: "could" };
}

export function ScheduleCard({ schedule }) {
  const net = schedule.total_income - schedule.total_cost;
  const tasks = schedule.tasks || [];
  const lastIdx = tasks.length - 1;

  return (
    <div className="sd-timeline sd-slide-in">
      <div className="sd-timeline-header">
        {schedule.summary}
      </div>

      <div className="sd-timeline-body">
        {tasks.map((task, i) => {
          const meta = getActionMeta(task.action);
          const pri = getPriorityMeta(task.priority);
          return (
            <div key={i} className="sd-tl-node">
              <div className="sd-tl-rail">
                <div className={`sd-tl-icon sd-pri-${pri.cls}`}>
                  {meta.icon}
                </div>
                {i < lastIdx && <div className="sd-tl-line" />}
              </div>
              <div className="sd-tl-content">
                <div className="sd-tl-desc">{task.description}</div>
                <div className="sd-tl-tags">
                  <span className="sd-tl-tag sd-tag-time">{task.time_cost}h</span>
                  {task.cost > 0 && (
                    <span className="sd-tl-tag sd-tag-cost">-{task.cost}g</span>
                  )}
                  {task.income > 0 && (
                    <span className="sd-tl-tag sd-tag-income">+{task.income}g</span>
                  )}
                  <span className={`sd-tl-tag sd-tag-pri sd-pri-${pri.cls}`}>
                    {pri.label}
                  </span>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="sd-timeline-footer">
        <span className="sd-tl-total-time">{schedule.total_time}h</span>
        <span className="sd-tl-total-cost">支出 {schedule.total_cost}g</span>
        <span className="sd-tl-total-income">收入 {schedule.total_income}g</span>
        <span
          className="sd-tl-net"
          style={{ color: net >= 0 ? "var(--sd-green-d)" : "var(--sd-red)" }}
        >
          净 {net >= 0 ? "+" : ""}{net}g
        </span>
      </div>

      {schedule.notes && schedule.notes.length > 0 && (
        <div className="sd-timeline-notes">
          {schedule.notes.map((n, i) => (
            <div key={i} className="sd-tl-note">! {n}</div>
          ))}
        </div>
      )}
    </div>
  );
}
