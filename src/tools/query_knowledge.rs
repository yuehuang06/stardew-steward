use crate::knowledge::KnowledgeBase;
use std::sync::Mutex;

pub fn execute(kb: &Mutex<KnowledgeBase>, keyword: &str) -> anyhow::Result<String> {
    let kb = kb.lock().map_err(|e| anyhow::anyhow!("知识库锁异常: {}", e))?;
    kb.search(keyword)
}
