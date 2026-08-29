use std::sync::Arc;

use crate::config::Config;
use crate::solver::schedule::DailySchedule;
use crate::tools::{ToolRegistry, ToolCallRequest};
use crate::usage::UsageTracker;
use crate::validator::Validator;
use super::message::Message;
use super::progress::ProgressReporter;

pub struct Agent {
    config: Config,
    client: reqwest::Client,
    tools: ToolRegistry,
    usage: UsageTracker,
    reporter: Arc<dyn ProgressReporter>,
    history: Vec<Message>,
    system_prompt: String,
    validator: Option<Validator>,
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
        }
    }

    pub fn set_validator(&mut self, v: Validator) {
        self.validator = Some(v);
    }

    pub fn usage_summary(&self) -> String {
        self.usage.summary()
    }

    /// Agent 主循环
    pub async fn run(&mut self, user_message: &str) -> anyhow::Result<String> {
        self.history.push(Message::user(user_message));

        for _step in 0..self.config.agent.max_steps {
            if self.reporter.is_interrupted() {
                return Ok("(用户已打断)".into());
            }

            self.reporter.on_step("正在思考...");
            let response = self.call_llm().await?;
            self.reporter.on_done();

            if let Some(tool_call) = response.tool_call {
                if !response.assistant_content.is_empty() {
                    println!("  💭 {}", response.assistant_content);
                }
                let tool_desc = describe_tool_call(&tool_call);
                let tool_msg = Message::assistant_with_tool_calls(
                    &response.assistant_content,
                    response.tool_calls_json.clone(),
                );
                self.history.push(tool_msg);

                self.reporter.on_step(&format!("正在执行: {}", tool_desc));
                let result = self.tools.execute(&tool_call).await?;
                self.reporter.on_done();

                self.history.push(Message::tool(&result, &response.tool_call_id));
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

                    // 校验通过，渲染成 Markdown
                    let rendered = crate::solver::schedule::render(&schedule);
                    self.history.push(Message::assistant(text));
                    return Ok(rendered);
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

    async fn call_llm(&mut self) -> anyhow::Result<LlmResponse> {
        let messages_json: Vec<serde_json::Value> = self.history.iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect();

        let body = serde_json::json!({
            "model": self.config.model.model,
            "messages": messages_json,
            "tools": self.tools.tool_definitions(),
            "tool_choice": "auto",
        });

        let endpoint = if self.config.model.endpoint.ends_with("/chat/completions") {
            self.config.model.endpoint.clone()
        } else {
            format!("{}/chat/completions", self.config.model.endpoint.trim_end_matches('/'))
        };

        let resp = self.client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.model.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let resp_text = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("LLM API 返回错误 {} ({}): {}", status, endpoint, resp_text);
        }

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| anyhow::anyhow!("解析 LLM 响应失败: {} | 原始响应: {}", e, &resp_text[..resp_text.len().min(500)]))?;

        let usage = &resp_json["usage"];
        if let Some(prompt) = usage["prompt_tokens"].as_u64() {
            if let Some(completion) = usage["completion_tokens"].as_u64() {
                self.usage.record(prompt, completion);
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

/// 从可能包含 ```json ... ``` 的文本中提取 JSON
fn extract_json(text: &str) -> Option<&str> {
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

/// 从工具调用参数中提取关键信息，生成人类可读的描述
fn describe_tool_call(call: &ToolCallRequest) -> String {
    match call.name.as_str() {
        "read_save" => "读取存档".to_string(),
        "query_knowledge" => {
            let kw = call.arguments["keyword"].as_str().unwrap_or("");
            format!("查询知识库: {}", kw)
        }
        "solve_schedule" => {
            let tasks = call.arguments["tasks"].as_array().map(|a| a.len()).unwrap_or(0);
            format!("求解日程（{}项任务）", tasks)
        }
        _ => call.name.clone(),
    }
}

struct LlmResponse {
    text: String,
    assistant_content: String,
    tool_call: Option<ToolCallRequest>,
    tool_call_id: String,
    tool_calls_json: serde_json::Value,
}
