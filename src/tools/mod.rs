pub mod read_save;
pub mod query_knowledge;
pub mod solve_schedule;

pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

struct ToolDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
    handler: Box<dyn Fn(serde_json::Value) -> anyhow::Result<String> + Send + Sync>,
}

pub struct ToolRegistry {
    tools: Vec<ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(
        &mut self,
        name: &str,
        description: &str,
        parameters: serde_json::Value,
        handler: impl Fn(serde_json::Value) -> anyhow::Result<String> + Send + Sync + 'static,
    ) {
        self.tools.push(ToolDef {
            name: name.into(),
            description: description.into(),
            parameters,
            handler: Box::new(handler),
        });
    }

    pub async fn execute(&self, call: &ToolCallRequest) -> anyhow::Result<String> {
        let tool = self.tools.iter()
            .find(|t| t.name == call.name)
            .ok_or_else(|| anyhow::anyhow!("未知工具: {}", call.name))?;
        (tool.handler)(call.arguments.clone())
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
