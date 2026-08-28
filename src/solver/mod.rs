// 规划求解器: 将"今天该干嘛"建模为约束优化，用 Rust 确定性求解
//
// 输入: GameState + 知识库数据
// 输出: 排好的日程（任务序列）
//
// LLM 给"人情味"的调整，求解器给"数学最优"的基线

pub mod task;
pub mod greedy;

pub use task::{Task, Schedule};
