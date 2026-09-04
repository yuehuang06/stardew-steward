use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub model: ModelConfig,
    pub save: SaveConfig,
    pub agent: AgentConfig,
    pub knowledge: KnowledgeConfig,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ModelConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub context_length: usize,
    pub thinking_mode: bool,
    pub price_input: f64,
    pub price_output: f64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SaveConfig {
    pub path: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    pub token_budget: u64,
    pub max_steps: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct KnowledgeConfig {
    pub db_path: String,
}

pub fn load() -> anyhow::Result<Config> {
    let candidates = [
        "config.toml".to_string(),
        "../config.toml".to_string(),
        format!("{}/config.toml", env!("CARGO_MANIFEST_DIR")),
    ];
    let text = candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "找不到 config.toml（尝试过: {}）",
                candidates.join(", ")
            )
        })?;
    let mut config: Config = toml::from_str(&text)?;

    if let Ok(key) = std::env::var("API_KEY") {
        if !key.is_empty() {
            config.model.api_key = key;
        }
    }

    let root = env!("CARGO_MANIFEST_DIR");
    if !std::path::Path::new(&config.knowledge.db_path).is_absolute() {
        config.knowledge.db_path = format!("{}/{}", root, config.knowledge.db_path);
    }

    Ok(config)
}

pub fn save(config: &Config) -> anyhow::Result<()> {
    let path = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let text = toml::to_string_pretty(config)?;
    std::fs::write(&path, text)?;
    Ok(())
}
