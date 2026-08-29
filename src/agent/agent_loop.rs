use std::sync::Arc;

use crate::config::Config;
use crate::tools::{ToolRegistry, ToolCallRequest};
use crate::usage::UsageTracker;
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
        }
    }

    pub fn usage_summary(&self) -> String {
        self.usage.summary()
    }

    /// Agent 主循环
    pub async fn run(&mut self, user_message: &str) -> anyhow::Result<String> {
        self.history.push(Message::user(user_message));

        for step in 0..self.config.agent.max_steps {
            if self.reporter.is_interrupted() {
                return Ok("(用户已打断)".into());
            }

            self.reporter.on_step("正在思考...");
            let response = self.call_llm().await?;
            self.reporter.on_done();

            if let Some(tool_call) = response.tool_call {
                let tool_msg = Message::assistant_with_tool_calls(
                    &response.assistant_content,
                    response.tool_calls_json.clone(),
                );
                self.history.push(tool_msg);

                self.reporter.on_step(&format!("正在执行工具: {}", tool_call.name));
                let result = self.tools.execute(&tool_call).await?;
                self.reporter.on_done();

                self.history.push(Message::tool(&result, &response.tool_call_id));
                continue;
            }

            self.history.push(Message::assistant(&response.text));
            return Ok(response.text);
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
                //记录了一则对话的usage
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

struct LlmResponse {
    text: String,
    assistant_content: String,
    tool_call: Option<ToolCallRequest>,
    tool_call_id: String,
    tool_calls_json: serde_json::Value,
}
