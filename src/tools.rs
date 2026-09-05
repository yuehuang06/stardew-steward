pub mod read_save;
pub mod query_knowledge;
pub mod solve_schedule;
pub mod fetch_wiki;
pub mod crop_advisor;
pub mod gift_finder;

use std::future::Future;
use std::pin::Pin;

pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

type AsyncHandler = Box<dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>> + Send + Sync>;

struct ToolDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
    handler: AsyncHandler,
}

pub struct ToolRegistry {
    tools: Vec<ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register<F, Fut>(
        &mut self,
        name: &str,
        description: &str,
        parameters: serde_json::Value,
        handler: F,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<String>> + Send + 'static,
    {
        self.tools.push(ToolDef {
            name: name.into(),
            description: description.into(),
            parameters,
            handler: Box::new(move |args| Box::pin(handler(args))),
        });
    }

    pub async fn execute(&self, call: &ToolCallRequest) -> anyhow::Result<String> {
        let tool = self.tools.iter()
            .find(|t| t.name == call.name)
            .ok_or_else(|| anyhow::anyhow!("未知工具: {}", call.name))?;
        (tool.handler)(call.arguments.clone()).await
    }

    pub fn tool_definitions(&self) -> Vec<serde_json::Value> {
        self.tools.iter().map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        }).collect()
    }
}
