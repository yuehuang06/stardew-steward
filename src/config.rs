use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub save: SaveConfig,
    pub agent: AgentConfig,
    pub knowledge: KnowledgeConfig,
}

#[derive(Deserialize)]
pub struct ModelConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub context_length: usize,
    pub thinking_mode: bool,
    pub price_input: f64,
    pub price_output: f64,
}

#[derive(Deserialize)]
pub struct SaveConfig {
    pub path: String,
}

#[derive(Deserialize)]
pub struct AgentConfig {
    pub token_budget: u64,
    pub max_steps: u32,
}

#[derive(Deserialize)]
pub struct KnowledgeConfig {
    pub db_path: String,
}

pub fn load() -> anyhow::Result<Config> {
    let text = std::fs::read_to_string("config.toml")
        .or_else(|_| std::fs::read_to_string("config.toml.example"))?;
    let mut config: Config = toml::from_str(&text)?;

    // .env 中的 API_KEY 优先
    if let Ok(key) = std::env::var("API_KEY") {
        if !key.is_empty() {
            config.model.api_key = key;
        }
    }
    Ok(config)
}
