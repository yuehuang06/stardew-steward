// R5: 会话持久化 — 保存/加载对话上下文
use serde::{Deserialize, Serialize};
use super::message::Message;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedSession {
    pub saved_at: u64,
    pub message_count: usize,
    pub messages: Vec<Message>,
}

pub fn sessions_dir() -> &'static str {
    "sessions"
}

pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// unix 秒 → "YYYY-MM-DD HH:MM:SS"（UTC）
pub fn format_time(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mth = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe as i64 + era * 400 + if mth <= 2 { 1 } else { 0 };

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mth, d, hh, mm, ss)
}

/// 列出所有已保存会话: (名称, 保存时间, 消息数)
pub fn list_sessions() -> Vec<(String, u64, usize)> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(sessions_dir()) {
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().map(|x| x == "json").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(s) = serde_json::from_str::<SavedSession>(&text) {
                        out.push((name, s.saved_at, s.message_count));
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}
