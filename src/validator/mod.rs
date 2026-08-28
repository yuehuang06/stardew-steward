// 校验闭环: LLM 输出建议后，用硬编码游戏规则校验合法性
//
// 校验失败 → 自动拼错误信息回 Prompt → LLM 重试
// 这是"固化流程"的核心: 确定性保障，不依赖 LLM 运气

pub struct Validator {
    money: i32,
    season: String,
    time_budget: f32,
}

impl Validator {
    pub fn new(money: i32, season: &str, time_budget: f32) -> Self {
        Self { money, season: season.into(), time_budget }
    }

    /// 校验 LLM 输出的建议 JSON，返回错误列表（空=通过）
    pub fn check(&self, advice: &serde_json::Value) -> Vec<String> {
        let mut errors = Vec::new();

        // 1. 资金检查
        if let Some(cost) = advice["seed_cost"].as_i64() {
            if cost as i32 > self.money {
                errors.push(format!(
                    "推荐的种子总价 {}g 超过玩家资金 {}g",
                    cost, self.money
                ));
            }
        }

        // 2. 时长检查
        if let Some(total) = advice["total_time"].as_f64() {
            if total as f32 > self.time_budget {
                errors.push(format!(
                    "日程总时长 {:.1}h 超过一天可用 {:.1}h",
                    total, self.time_budget
                ));
            }
        }

        // 3. 季节检查（硬编码游戏常识）
        if let Some(crops) = advice["recommended_crops"].as_array() {
            let winter_crops = ["蓝莓", "玉米", "辣椒", "茄子"];
            for crop in crops {
                if let Some(name) = crop.as_str() {
                    if self.season == "winter" && winter_crops.contains(&name) {
                        errors.push(format!("{}不能在冬天种植", name));
                    }
                }
            }
        }

        errors
    }
}
