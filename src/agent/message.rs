use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self { role: Role::System, content: content.into(), tool_call_id: None, tool_calls: None }
    }
    pub fn user(content: &str) -> Self {
        Self { role: Role::User, content: content.into(), tool_call_id: None, tool_calls: None }
    }
    pub fn assistant(content: &str) -> Self {
        Self { role: Role::Assistant, content: content.into(), tool_call_id: None, tool_calls: None }
    }
    pub fn tool(content: &str, tool_call_id: &str) -> Self {
        Self { role: Role::Tool, content: content.into(), tool_call_id: Some(tool_call_id.into()), tool_calls: None }
    }
    pub fn assistant_with_tool_calls(
        content: &str,
        tool_calls: serde_json::Value,
    ) -> Self {
        Self { role: Role::Assistant, content: content.into(), tool_call_id: None, tool_calls: Some(tool_calls) }
    }
}
