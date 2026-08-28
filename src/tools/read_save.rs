use crate::parser::GameState;

/// 工具: read_save
/// 读取本地存档，返回结构化游戏状态 JSON
pub fn execute(path: &str) -> anyhow::Result<String> {
    let state = crate::parser::parse(std::path::Path::new(path))?;
    let json = serde_json::to_string_pretty(&state)?;
    Ok(json)
}
