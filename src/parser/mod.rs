// 存档解析器: XML 存档 → 结构化 GameState

pub mod save;
pub mod state;

pub use save::parse;
pub use state::GameState;
