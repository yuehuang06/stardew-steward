// Token 用量与价格统计 (R6)
//
// 每次调用 LLM API 后，从 response.usage 读取 token 数
// 按配置价格换算成本，累计统计，达到预算自动中断

use crate::config::Config;

pub struct UsageTracker {
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_cost: f64,
    budget: u64,
    price_input: f64,   // per 1K tokens
    price_output: f64,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost: 0.0,
            budget: u64::MAX,
            price_input: 0.0,
            price_output: 0.0,
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            budget: config.agent.token_budget,
            price_input: config.model.price_input,
            price_output: config.model.price_output,
            ..Self::new()
        }
    }

    pub fn record(&mut self, input: u64, output: u64) {
        self.total_input_tokens += input;
        self.total_output_tokens += output;
        self.total_cost += (input as f64 / 1000.0) * self.price_input
            + (output as f64 / 1000.0) * self.price_output;
    }

    pub fn over_budget(&self) -> bool {
        self.total_input_tokens + self.total_output_tokens >= self.budget
    }

    pub fn summary(&self) -> String {
        format!(
            "Token: {} in / {} out | 成本: ¥{:.4} | 预算: {} tokens",
            self.total_input_tokens,
            self.total_output_tokens,
            self.total_cost,
            self.budget,
        )
    }
}
