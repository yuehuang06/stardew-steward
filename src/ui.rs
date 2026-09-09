use stardew_steward::agent::ProgressReporter;

pub struct CliReporter;

impl CliReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for CliReporter {
    fn on_step(&self, message: &str) {
        println!("  → {}", message);
    }

    fn on_done(&self) {
        println!("  ✓ 完成");
    }

    fn on_thinking(&self, message: &str) {
        println!("  💭 {}", message);
    }

    fn on_error(&self, message: &str) {
        println!("  ✗ {}", message);
    }

    fn on_heartbeat(&self, message: &str) {
        println!("  ⏳ {}", message);
    }
}

pub fn print_welcome() {
    println!();
    println!("🌾 星露谷农场管家 v0.1.0");
    println!("──────────────────────────────────");
    println!("输入你的问题，或输入 /quit 退出");
    println!("──────────────────────────────────");
}

pub fn print_usage(summary: &str) {
    println!("📊 {}", summary);
}

pub fn print_response(text: &str) {
    println!();
    termimad::print_text(&render_reply(text));
    println!();
}

/// 组装最终渲染用的 markdown：
/// 能提取出日程 JSON → 前置说明 + 表格 + 后置说明；否则原样（剥围栏）
fn render_reply(text: &str) -> String {
    if let Some(json_str) = stardew_steward::agent::extract_json(text) {
        match serde_json::from_str::<stardew_steward::solver::schedule::DailySchedule>(json_str) {
            Ok(schedule) => {
                let pos = json_str.as_ptr() as usize - text.as_ptr() as usize;
                let mut before = text[..pos].trim_end().to_string();
                let mut after = text[pos + json_str.len()..].trim_start().to_string();
                // 剥离紧邻 JSON 的围栏标记（```json 开头 / ``` 闭合），
                // 否则拼接后表格被包进围栏，termimad 会当成代码块渲染
                for marker in ["```json", "```JSON", "```"] {
                    if let Some(s) = before.strip_suffix(marker) {
                        before = s.trim_end().to_string();
                        break;
                    }
                }
                if let Some(s) = after.strip_prefix("```") {
                    after = s.trim_start().to_string();
                }
                let mut md = String::new();
                if !before.is_empty() {
                    md.push_str(strip_markdown_fence(&before));
                    md.push_str("\n\n");
                }
                md.push_str(&stardew_steward::solver::schedule::render(&schedule));
                if !after.is_empty() {
                    md.push_str("\n\n");
                    md.push_str(strip_markdown_fence(&after));
                }
                return md;
            }
            Err(e) => {
                if std::env::var("SS_DEBUG").is_ok() {
                    eprintln!("[SS_DEBUG] 日程 JSON 解析失败: {:?}\n提取内容前 300 字: {}", e, &json_str.chars().take(300).collect::<String>());
                }
            }
        }
    }
    strip_markdown_fence(text).to_string()
}

/// 剥离包裹整个回答的 markdown 围栏
/// LLM 有时把 markdown 表格包进 ```markdown ... ```，termimad 会把围栏内容当
/// 代码原样输出（用户看到表格源码）——这里把外层围栏拆掉再渲染
fn strip_markdown_fence(text: &str) -> &str {
    let t = text.trim();
    let Some(rest) = t.strip_prefix("```") else {
        return text;
    };
    // 围栏首行只允许 ``` / ```markdown / ```md（真代码块保持原样）
    let mut lines = rest.split('\n');
    let Some(first) = lines.next() else {
        return text;
    };
    let lang = first.trim();
    if !lang.is_empty() && lang != "markdown" && lang != "md" {
        return text;
    }
    let Some(body) = rest.strip_prefix(first) else {
        return text;
    };
    let body = body.trim_start_matches('\n');
    let Some(body) = body.strip_suffix("```") else {
        return text;
    };
    body.trim_end_matches('\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEDULE_JSON: &str = "{\"summary\":\"s\",\"tasks\":[{\"action\":\"water\",\"description\":\"浇水\",\"time_cost\":0.5,\"cost\":0,\"income\":0,\"priority\":\"must\"}],\"total_time\":0.5,\"total_cost\":0,\"total_income\":0,\"notes\":[]}";

    #[test]
    fn fenced_schedule_renders_as_table() {
        let text = format!("开头说明\n\n```json\n{}\n```\n\n收尾补充", SCHEDULE_JSON);
        let md = render_reply(&text);
        assert!(md.contains("| # |"), "应包含表格，实际: {}", md);
        assert!(md.contains("开头说明") && md.contains("收尾补充"), "前后说明应保留");
        // 回归: 拼接结果不能残留围栏标记，否则 termimad 把表格当代码块渲染
        assert!(!md.contains("```"), "拼接 md 不应残留围栏标记，实际: {}", md);
    }

    #[test]
    fn bare_schedule_renders_as_table() {
        let md = render_reply(SCHEDULE_JSON);
        assert!(md.contains("| # |"), "裸 JSON 也应渲染表格");
    }

    #[test]
    fn markdown_fence_around_table_is_stripped() {
        let md = strip_markdown_fence("```markdown\n| a | b |\n|---|---|\n| 1 | 2 |\n```");
        assert!(md.starts_with("| a |"));
    }

    #[test]
    fn code_fence_is_kept() {
        let md = strip_markdown_fence("```rust\nfn main() {}\n```");
        assert!(md.contains("```rust"), "真代码块不应被剥离");
    }
}
