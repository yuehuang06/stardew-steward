// R5: 会话持久化 — 保存/加载对话上下文
use serde::{Deserialize, Serialize};
use super::message::{Message, Role};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedSession {
    pub saved_at: u64,
    pub message_count: usize,
    /// 用户交互轮数（排除 system/tool 消息）
    pub interaction_rounds: usize,
    pub messages: Vec<Message>,
    /// 该会话累计的 token 用量和花费
    #[serde(default)]
    pub usage: SessionUsage,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SessionUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
}

pub fn sessions_dir() -> String {
    // 相对路径解析到项目根目录，保证 cargo tauri dev（CWD=src-tauri/）也正确
    let dir = "sessions";
    if std::path::Path::new(dir).is_absolute() {
        dir.to_string()
    } else {
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), dir)
    }
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

/// 生成自动保存的会话名: "auto_0910_1430"（月日_时分）
pub fn auto_name() -> String {
    let secs = now_secs();
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (hh, mm) = (rem / 3600, (rem % 3600) / 60);

    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mth = if mp < 10 { mp + 3 } else { mp - 9 };

    format!("auto_{:02}{:02}_{:02}{:02}", mth, d, hh, mm)
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

/// 统计交互轮数（user 消息数）
pub fn count_rounds(messages: &[Message]) -> usize {
    messages.iter().filter(|m| matches!(m.role, Role::User)).count()
}

/// 列出所有已保存会话: (名称, 保存时间, 消息数, 交互轮数)
pub fn list_sessions() -> Vec<(String, u64, usize, usize)> {
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
                        out.push((name, s.saved_at, s.message_count, s.interaction_rounds));
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}
