/**
 * Try to extract a DailySchedule JSON object from assistant message text.
 * Returns parsed object or null if not a schedule.
 */
export function tryParseSchedule(text) {
  if (!text || typeof text !== "string") return null;

  let raw = text.trim();

  // Strip ```json ... ``` fence
  const fence = raw.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (fence) {
    raw = fence[1].trim();
  }

  // Must look like a JSON object with summary + tasks
  if (!raw.startsWith("{")) return null;

  try {
    const obj = JSON.parse(raw);
    if (
      obj &&
      typeof obj.summary === "string" &&
      Array.isArray(obj.tasks)
    ) {
      return obj;
    }
  } catch {
    return null;
  }

  return null;
}
