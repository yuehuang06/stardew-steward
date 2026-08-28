// 游戏知识库: wiki 数据索引 + 关键词检索 (RAG)

pub struct KnowledgeBase {
    // TODO: rusqlite 连接
    // 表结构: crops(名称, 季节, 生长天数, 收益, 是否复收...)
    //         npcs(名称, 生日, 最爱礼物, 喜欢礼物...)
    //         fish(名称, 季节, 天气, 时间, 地点...)
}

impl KnowledgeBase {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        // TODO: 打开/创建 SQLite，建表，导入 data/*.json
        todo!("P2 阶段实现")
    }

    pub fn search(&self, keyword: &str) -> Vec<String> {
        // TODO: 关键词检索 → 返回相关知识片段
        todo!()
    }
}
