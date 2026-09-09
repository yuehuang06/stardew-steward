use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::Config;
use crate::solver::schedule::DailySchedule;
use crate::tools::{ToolRegistry, ToolCallRequest};
use crate::usage::UsageTracker;
use crate::validator::Validator;
use super::message::{Message, Role};
use super::progress::ProgressReporter;
use super::session::{self, SavedSession};

pub struct Agent {
    config: Config,
    client: reqwest::Client,
    tools: ToolRegistry,
    usage: UsageTracker,
    reporter: Arc<dyn ProgressReporter>,
    history: Vec<Message>,
    system_prompt: String,
    validator: Option<Validator>,
    interrupt_flag: Option<Arc<AtomicBool>>,
    session_title: Option<String>,
}

impl Agent {
    pub fn new(
        config: Config,
        tools: ToolRegistry,
        reporter: Arc<dyn ProgressReporter>,
        system_prompt: String,
    ) -> Self {
        let client = reqwest::Client::new();
        let usage = UsageTracker::from_config(&config);
        Self {
            config,
            client,
            tools,
            usage,
            reporter,
            history: vec![Message::system(&system_prompt)],
            system_prompt,
            validator: None,
            interrupt_flag: None,
            session_title: None,
        }
    }

    pub fn set_validator(&mut self, v: Validator) {
        self.validator = Some(v);
    }

    /// 新建对话：清空历史（保留 system prompt），重置 session 标题
    pub fn new_session(&mut self) {
        self.history = vec![Message::system(&self.system_prompt)];
        self.session_title = None;
        // 重置 per-session 用量统计
        self.usage = UsageTracker::from_config(&self.config);
    }

    /// R4: 注入打断标记（由 main 的 Ctrl-C 信号任务置位）
    pub fn set_interrupt_flag(&mut self, flag: Arc<AtomicBool>) {
        self.interrupt_flag = Some(flag);
    }

    fn interrupted(&self) -> bool {
        self.interrupt_flag
            .as_ref()
            .map(|f| f.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    pub fn usage_summary(&self) -> String {
        self.usage.summary()
    }

    /// 简短用量: "¥0.05 · 14K/200K tok"（用于 prompt 前缀实时显示）
    /// 获取当前会话标题
    pub fn session_title(&self) -> String {
        self.session_title.clone().unwrap_or_else(|| {
            // 没有标题时，用第一条用户消息截取前 20 字
            self.history.iter()
                .find(|m| matches!(m.role, Role::User))
                .map(|m| {
                    let t = strip_reminder(&m.content).trim();
                    if t.chars().count() <= 20 { t.to_string() }
                    else { format!("{}…", t.chars().take(20).collect::<String>()) }
                })
                .unwrap_or_else(|| "untitled".to_string())
        })
    }

    pub fn usage_brief(&self) -> String {
        let used = self.usage.total_input_tokens() + self.usage.total_output_tokens();
        let budget = self.usage.budget();
        let cost = self.usage.total_cost();
        format!("¥{:.4} · {}K/{}K tok", cost, used / 1000, budget / 1000)
    }

    pub fn usage_detail(&self) -> (u64, u64, u64, f64) {
        (
            self.usage.total_input_tokens(),
            self.usage.total_output_tokens(),
            self.usage.budget(),
            self.usage.total_cost(),
        )
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_token_budget(&mut self, budget: u64) {
        self.config.agent.token_budget = budget;
        self.usage.set_budget(budget);
    }

    pub fn update_model_config(&mut self, model: crate::config::ModelConfig) {
        self.config.model = model;
        self.usage = UsageTracker::from_config(&self.config);
    }

    /// R5: 保存当前会话到 sessions/<name>.json，返回文件路径
    pub fn save_session(&self, name: &str) -> anyhow::Result<String> {
        let (input, output, _, cost) = self.usage_detail();
        let session = SavedSession {
            saved_at: session::now_secs(),
            message_count: self.history.len(),
            interaction_rounds: session::count_rounds(&self.history),
            messages: self.history.clone(),
            usage: session::SessionUsage {
                input_tokens: input,
                output_tokens: output,
                cost,
            },
        };
        std::fs::create_dir_all(session::sessions_dir())?;
        let path = format!(
            "{}/{}.json",
            session::sessions_dir(),
            session::sanitize_name(name)
        );
        std::fs::write(&path, serde_json::to_string_pretty(&session)?)?;
        Ok(path)
    }

    /// R5: 自动保存（用会话标题命名，标题取自用户首条消息）
    pub fn auto_save_session(&self) -> Option<String> {
        let title = self.session_title();
        match self.save_session(&title) {
            Ok(path) => Some(path),
            Err(_) => None,
        }
    }

    /// R5: 加载会话，替换当前历史，返回(消息数, 交互轮数, 会话用量)
    pub fn load_session(&mut self, name: &str) -> anyhow::Result<(usize, usize, session::SessionUsage)> {
        let path = format!(
            "{}/{}.json",
            session::sessions_dir(),
            session::sanitize_name(name)
        );
        let text = std::fs::read_to_string(&path)
            .map_err(|_| anyhow::anyhow!("会话「{}」不存在（用 /sessions 查看已保存的会话）", name))?;
        let s: SavedSession = serde_json::from_str(&text)?;
        let rounds = s.interaction_rounds;
        let usage = s.usage.clone();
        self.history = s.messages;
        // 重置 per-session 用量统计，用加载的会话用量初始化
        self.usage = UsageTracker::from_config(&self.config);
        self.usage.restore(usage.input_tokens, usage.output_tokens, usage.cost);
        Ok((self.history.len(), rounds, usage))
    }

    /// R5: 删除指定会话文件
    pub fn delete_session(name: &str) -> anyhow::Result<()> {
        let path = format!(
            "{}/{}.json",
            session::sessions_dir(),
            session::sanitize_name(name)
        );
        std::fs::remove_file(&path)
            .map_err(|_| anyhow::anyhow!("会话「{}」不存在", name))?;
        Ok(())
    }

    /// 返回前端展示用的聊天消息（仅 user / assistant，跳过 system / tool）
    pub fn chat_messages(&self) -> Vec<(String, String)> {
        self.history
            .iter()
            .filter(|m| matches!(m.role, Role::User | Role::Assistant))
            .map(|m| {
                let role = match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    _ => unreachable!(),
                };
                (role.to_string(), strip_reminder(&m.content).to_string())
            })
            .collect()
    }

    /// R5: 导出 Agent 完整工作轨迹（含工具调用，非黑盒）
    pub fn trajectory(&self) -> Vec<String> {
        self.history
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "系统",
                    Role::User => "用户",
                    Role::Assistant => "Agent",
                    Role::Tool => "工具",
                };
                let mut line = format!("[{}] {}", role, m.content.replace('\n', " "));
                if let Some(tc) = &m.tool_calls {
                    let names: Vec<String> = tc
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|c| c["function"]["name"].as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    line.push_str(&format!("（调用工具: {}）", names.join(", ")));
                }
                if line.chars().count() > 200 {
                    line = line.chars().take(200).collect();
                    line.push_str("…");
                }
                line
            })
            .collect()
    }

    /// Agent 主循环
    pub async fn run(&mut self, user_message: &str) -> anyhow::Result<String> {
        // 用户消息保持干净（会进历史/标题/展示），提醒用独立 system 消息注入
        self.history.push(Message::user(user_message));
        self.history.push(Message::system(
            "系统提醒：玩家可能已推进游戏进度，存档状态可能已变化。请重新调用 read_save 工具获取最新存档，不要复用上下文中的旧存档数据。",
        ));

        for _step in 0..self.config.agent.max_steps {
            if self.interrupted() {
                return Ok("(用户已打断)".into());
            }

            self.reporter.on_step("正在思考...");
            // R4: select! 让 LLM 调用可被 Ctrl-C 打断
            enum LlmStep {
                Done(anyhow::Result<LlmResponse>),
                Stop,
            }
            let flag = self.interrupt_flag.clone();
            let step = match flag {
                Some(f) => {
                    // 克隆 reporter 给心跳协程，避免与 call_llm 的 &mut self 借用冲突
                    let reporter = Arc::clone(&self.reporter);
                    tokio::select! {
                        r = self.call_llm() => LlmStep::Done(r),
                        _ = poll_flag(f, reporter) => LlmStep::Stop,
                    }
                }
                None => LlmStep::Done(self.call_llm().await),
            };
            self.reporter.on_done();

            let response = match step {
                LlmStep::Done(Ok(resp)) => resp,
                LlmStep::Done(Err(e)) => {
                    self.reporter.on_error("调用失败");
                    return Err(e);
                }
                LlmStep::Stop => {
                    self.reporter.on_error("已打断");
                    return Ok("(用户已打断本轮任务)".into());
                }
            };

            if let Some(tool_call) = response.tool_call {
                if !response.assistant_content.is_empty() {
                    self.reporter.on_thinking(&response.assistant_content);
                }
                let tool_desc = describe_tool_call(&tool_call);
                let tool_msg = Message::assistant_with_tool_calls(
                    &response.assistant_content,
                    response.tool_calls_json.clone(),
                );
                self.history.push(tool_msg);

                self.reporter.on_step(&tool_desc);
                let result = self.tools.execute(&tool_call).await?;
                self.reporter.on_done();

                self.history.push(Message::tool(&result, &response.tool_call_id));

                if self.interrupted() {
                    return Ok("(用户已打断)".into());
                }
                continue;
            }

            // LLM 返回了文本——尝试解析为日程 JSON
            let text = response.text.trim();

            // 尝试提取 JSON（LLM 可能在 JSON 外面包了 ```json ... ```）
            let json_str = extract_json(text).unwrap_or(text);

            match serde_json::from_str::<DailySchedule>(json_str) {
                Ok(schedule) => {
                    // 校验
                    if let Some(ref validator) = self.validator {
                        let errors = validator.check(&schedule);
                        if !errors.is_empty() {
                            let error_msg = format!(
                                "你的日程未通过校验，请修正后重新输出 JSON：\n{}",
                                errors.iter()
                                    .map(|e| format!("- {}", e))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            );
                            self.reporter.on_step("校验未通过，正在重试...");
                            self.reporter.on_done();
                            self.history.push(Message::assistant(text));
                            self.history.push(Message::user(&error_msg));
                            continue;
                        }
                    }

                    // 校验通过，返回原始 JSON 给前端渲染
                    self.history.push(Message::assistant(text));
                    return Ok(text.to_string());
                }
                Err(_) => {
                    // 不是 JSON，当普通文本回答返回
                    self.history.push(Message::assistant(text));
                    return Ok(text.to_string());
                }
            }
        }

        Ok("(达到最大步数，自动停止)".into())
    }

    fn build_messages(&self) -> Vec<serde_json::Value> {
        let ctx = self.config.model.context_length;
        if ctx == 0 {
            return self.history.iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
        }

        let reserved = 2000;
        let available = ctx.saturating_sub(reserved);
        let max_chars = available.saturating_mul(3);

        if self.history.is_empty() {
            return vec![];
        }

        let total: usize = self.history.iter()
            .map(|m| m.content.chars().count() + 50)
            .sum();
        if total <= max_chars {
            return self.history.iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
        }

        let system_chars = self.history[0].content.chars().count() + 50;
        let mut budget = max_chars.saturating_sub(system_chars);
        let mut kept: Vec<usize> = vec![0];

        let mut i = self.history.len();
        while i > 1 {
            let msg = &self.history[i - 1];
            let msg_chars = msg.content.chars().count() + 50;

            if matches!(msg.role, Role::Tool) && i > 2
                && self.history[i - 2].tool_calls.is_some()
            {
                let asst = &self.history[i - 2];
                let asst_chars = asst.content.chars().count() + 50;
                let tc_chars = asst.tool_calls.as_ref()
                    .map(|tc| serde_json::to_string(tc).unwrap_or_default().chars().count())
                    .unwrap_or(0);
                let pair = msg_chars + asst_chars + tc_chars;

                if budget >= pair {
                    budget -= pair;
                    kept.push(i - 2);
                    kept.push(i - 1);
                    i -= 2;
                } else {
                    break;
                }
            } else {
                if budget >= msg_chars {
                    budget -= msg_chars;
                    kept.push(i - 1);
                    i -= 1;
                } else {
                    break;
                }
            }
        }

        kept.sort();
        kept.iter()
            .map(|&idx| serde_json::to_value(&self.history[idx]).unwrap_or_default())
            .collect()
    }

    async fn call_llm(&mut self) -> anyhow::Result<LlmResponse> {
        let messages_json = self.build_messages();

        let mut body = serde_json::json!({
            "model": self.config.model.model,
            "messages": messages_json,
            "tools": self.tools.tool_definitions(),
            "tool_choice": "auto",
        });

        if self.config.model.thinking_mode {
            body["thinking"] = serde_json::json!({"type": "enabled"});
        }

        let endpoint = if self.config.model.endpoint.ends_with("/chat/completions") {
            self.config.model.endpoint.clone()
        } else {
            format!("{}/chat/completions", self.config.model.endpoint.trim_end_matches('/'))
        };

        // 网关类错误（429/502/503/504）和网络错误自动重试 — 共 3 次尝试
        const MAX_ATTEMPTS: u32 = 3;
        let mut attempt = 0u32;
        let (status, resp_text) = loop {
            attempt += 1;
            let sent = self.client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", self.config.model.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .timeout(std::time::Duration::from_secs(180))
                .send()
                .await;

            let outcome = match sent {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    let code = status.as_u16();
                    if code == 429 || code == 502 || code == 503 || code == 504 {
                        Err(format!("网关错误 {}（网关超时或平台过载）", code))
                    } else {
                        Ok((status, text))
                    }
                }
                Err(e) => Err(format!("网络错误: {}", e)),
            };

            match outcome {
                Ok(pair) => break pair,
                Err(reason) => {
                    if attempt >= MAX_ATTEMPTS {
                        anyhow::bail!(
                            "LLM API 连续 {} 次失败，最后原因: {} (endpoint: {})",
                            attempt, reason, endpoint
                        );
                    }
                    let wait_secs = attempt * 2;
                    self.reporter.on_step(&format!(
                        "{}，{} 秒后重试（第 {}/{} 次尝试）...",
                        reason, wait_secs, attempt + 1, MAX_ATTEMPTS
                    ));
                    tokio::time::sleep(std::time::Duration::from_secs(wait_secs as u64)).await;
                }
            }
        };

        if !status.is_success() {
            anyhow::bail!("LLM API 返回错误 {} ({}): {}", status, endpoint, resp_text);
        }

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| anyhow::anyhow!("解析 LLM 响应失败: {} | 原始响应: {}", e, &resp_text[..resp_text.len().min(500)]))?;

        let usage = &resp_json["usage"];
        if let Some(prompt) = usage["prompt_tokens"].as_u64() {
            if let Some(completion) = usage["completion_tokens"].as_u64() {
                self.usage.record(prompt, completion);
                // 每步实时上报用量
                self.reporter.on_usage(&self.usage_brief());
                if self.usage.over_budget() {
                    anyhow::bail!("Token 预算已用尽: {}", self.usage.summary());
                }
            }
        }

        let choice = &resp_json["choices"][0]["message"];
        let content = choice["content"].as_str().unwrap_or("").to_string();
        let finish_reason = resp_json["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("stop");

        if finish_reason == "tool_calls" {
            let tool_calls = &choice["tool_calls"];
            let first_call = &tool_calls[0];
            let call_id = first_call["id"].as_str().unwrap_or("unknown").to_string();
            let fn_name = first_call["function"]["name"].as_str().unwrap_or("").to_string();
            let fn_args_str = first_call["function"]["arguments"].as_str().unwrap_or("{}");

            let arguments: serde_json::Value = serde_json::from_str(fn_args_str)
                .unwrap_or(serde_json::json!({}));

            let tool_calls_json = tool_calls.clone();

            return Ok(LlmResponse {
                text: String::new(),
                assistant_content: content,
                tool_call: Some(ToolCallRequest { name: fn_name, arguments }),
                tool_call_id: call_id,
                tool_calls_json,
            });
        }

        Ok(LlmResponse {
            text: content,
            assistant_content: String::new(),
            tool_call: None,
            tool_call_id: String::new(),
            tool_calls_json: serde_json::Value::Null,
        })
    }
}

/// 从可能包含 ```json ... ``` 的文本中提取 JSON（围栏优先，退化为首个 { 到末个 }）
pub fn extract_json(text: &str) -> Option<&str> {
    if let Some(start) = text.find("```json") {
        let rest = &text[start + 7..];
        if let Some(end) = rest.find("```") {
            return Some(rest[..end].trim());
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(&text[start..=end]);
        }
    }
    None
}

/// 清洗旧版会话中拼进用户消息的系统提醒前缀
/// 旧格式: "[系统提醒: ...]\n实际消息"
fn strip_reminder(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("[系统提醒:") {
        if let Some(nl) = rest.find('\n') {
            return &rest[nl + 1..];
        }
    }
    content
}

/// 轮询打断标记，置位时返回（用于 select! 打断 LLM 调用）。
/// 长时间无响应时每 30 秒发一次心跳（CLI 打印防呆；GUI 不显示，保持原有进度步骤闪烁）
async fn poll_flag(flag: Arc<AtomicBool>, reporter: Arc<dyn ProgressReporter>) {
    let mut ticks: u64 = 0;
    loop {
        if flag.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        ticks += 1;
        if ticks % 60 == 0 {
            // 每 30 秒（60 个半秒 tick）报告一次
            reporter.on_heartbeat(&format!(
                "模型仍在生成，已等待 {} 秒（Ctrl-C 可打断）...",
                ticks / 2
            ));
        }
    }
}

/// 从工具调用参数中提取关键信息，生成人类可读的描述
fn describe_tool_call(call: &ToolCallRequest) -> String {
    match call.name.as_str() {
        "read_save" => "正在读取存档".to_string(),
        "query_knowledge" => {
            let kw = call.arguments["keyword"].as_str().unwrap_or("");
            format!("正在查询知识库: {}", kw)
        }
        "fetch_wiki" => {
            let kw = call.arguments["keyword"].as_str().unwrap_or("");
            if kw.is_empty() {
                "正在从 wiki 寻找更多知识".to_string()
            } else {
                format!("正在从 wiki 寻找: {}", kw)
            }
        }
        "solve_schedule" => {
            let tasks = call.arguments["tasks"].as_array().map(|a| a.len()).unwrap_or(0);
            format!("正在求解日程（{}项任务）", tasks)
        }
        "auto_schedule" => "正在自动生成日程".to_string(),
        "crop_advisor" => "正在分析作物收益".to_string(),
        "gift_finder" => "正在匹配送礼方案".to_string(),
        "farm_hand" => {
            let action = call.arguments["action"].as_str().unwrap_or("工作");
            let what = match action {
                "water_all" => "浇水",
                "harvest_all" => "收获",
                "clear_dead" => "清理枯株",
                "rollback_day" => "回档到昨天",
                _ => "工作",
            };
            format!("农场助手正在{}", what)
        }
        _ => format!("正在执行: {}", call.name),
    }
}

struct LlmResponse {
    text: String,
    assistant_content: String,
    tool_call: Option<ToolCallRequest>,
    tool_call_id: String,
    tool_calls_json: serde_json::Value,
}
