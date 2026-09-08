use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

/// 返回应用数据目录，首次运行时自动创建
pub fn app_data_dir() -> PathBuf {
    let dir = if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("stardew-steward")
    } else if cfg!(target_os = "macos") {
        dirs_home().join("Library/Application Support/stardew-steward")
    } else {
        let xdg = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            dirs_home().join(".config").to_string_lossy().into_owned()
        });
        PathBuf::from(xdg).join("stardew-steward")
    };
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

/// 自动检测星露谷物语存档路径
pub fn detect_save_path() -> Option<String> {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").ok()?;
        let base = PathBuf::from(&appdata).join("StardewValley/Saves");
        find_newest_save(&base)
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").ok()?;
        let base = PathBuf::from(&home)
            .join("Library/Application Support/Steam/steamapps/common/StardewValley/Saves");
        find_newest_save(&base)
    } else {
        // Linux/WSL
        let home = std::env::var("HOME").ok()?;
        // Try Proton path
        let base = PathBuf::from(&home)
            .join(".steam/steam/steamapps/compatdata/413150/pfx/drive_c/users/steamuser/AppData/Roaming/StardewValley/Saves");
        if let Some(p) = find_newest_save(&base) {
            return Some(p);
        }
        // Try WSL path
        let wsl = "/mnt/c/Users";
        if let Ok(entries) = std::fs::read_dir(wsl) {
            for entry in entries.flatten() {
                let base = entry.path().join("AppData/Roaming/StardewValley/Saves");
                if let Some(p) = find_newest_save(&base) {
                    return Some(p);
                }
            }
        }
        None
    }
}

/// 在 Saves 目录中找到最新角色存档
fn find_newest_save(base: &Path) -> Option<String> {
    let entries = std::fs::read_dir(base).ok()?;
    let mut newest: Option<(std::time::SystemTime, String)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        // Skip old backups
        if name.ends_with("_old") || name.starts_with('.') {
            continue;
        }
        // The save file has same name as the folder
        let save_file = path.join(&name);
        if !save_file.exists() {
            continue;
        }
        let mtime = entry.metadata().ok()
            .and_then(|m| m.modified().ok());
        if let Some(t) = mtime {
            match &newest {
                None => newest = Some((t, save_file.to_string_lossy().into_owned())),
                Some((old_t, _)) if t > *old_t => {
                    newest = Some((t, save_file.to_string_lossy().into_owned()));
                }
                _ => {}
            }
        } else {
            let p = save_file.to_string_lossy().into_owned();
            if newest.is_none() {
                newest = Some((std::time::SystemTime::now(), p));
            }
        }
    }
    newest.map(|(_, p)| p)
}

/// 首次运行时将内置 Ferris 存档写入 app data 目录
pub fn ensure_builtin_save() -> Option<String> {
    let dir = app_data_dir().join("saves").join("Rust");
    let save_file = dir.join("Rust");
    if save_file.exists() {
        return Some(save_file.to_string_lossy().into_owned());
    }
    std::fs::create_dir_all(&dir).ok()?;

    // 从编译进二进制的内置存档中提取
    let builtin_save = include_bytes!("../data/saves/Rust");
    let builtin_info = include_bytes!("../data/saves/SaveGameInfo");

    std::fs::write(&save_file, builtin_save).ok()?;
    std::fs::write(dir.join("SaveGameInfo"), builtin_info).ok()?;

    Some(save_file.to_string_lossy().into_owned())
}

pub fn load() -> anyhow::Result<Config> {
    let dir = app_data_dir();
    let config_path = dir.join("config.toml");

    if !config_path.exists() {
        let save_path = detect_save_path()
            .or_else(ensure_builtin_save)
            .unwrap_or_else(|| String::new());
        let default = default_config(&save_path);
        let text = toml::to_string_pretty(&default)?;
        std::fs::write(&config_path, text)?;

        // 同时写入一份带注释的示例文件供用户参考
        let example = format!(
            "# Stardew Steward 配置文件\n\
            # 此文件位于应用数据目录，可手动编辑也可在 GUI 设置面板修改\n\
            # 修改后重启应用生效\n\n\
            [model]\n\
            # API 配置 — 支持任意 OpenAI 兼容的 endpoint\n\
            # 切换到 OpenAI: endpoint = \"https://api.openai.com/v1\"\n\
            # 切换到本地: endpoint = \"http://localhost:11434/v1\"\n\
            endpoint = \"{}\"\n\
            api_key = \"在此填入你的 API Key\"\n\
            model = \"{}\"\n\
            context_length = {}\n\
            thinking_mode = {}\n\
            price_input = {}\n\
            price_output = {}\n\n\
            [save]\n\
            # 存档路径 — 留空则自动检测星露谷存档\n\
            path = \"{}\"\n\n\
            [agent]\n\
            # 每会话 Token 预算（达到上限自动中断）\n\
            token_budget = {}\n\
            max_steps = {}\n\n\
            [knowledge]\n\
            db_path = \"knowledge.db\"\n",
            default.model.endpoint, default.model.model, default.model.context_length,
            default.model.thinking_mode, default.model.price_input, default.model.price_output,
            save_path, default.agent.token_budget, default.agent.max_steps,
        );
        std::fs::write(dir.join("config.toml.example"), example).ok();
    }

    let text = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("读取配置失败: {}", e))?;
    let mut config: Config = toml::from_str(&text)?;

    if let Ok(key) = std::env::var("API_KEY") {
        if !key.is_empty() {
            config.model.api_key = key;
        }
    }

    // Resolve db_path to absolute
    if !Path::new(&config.knowledge.db_path).is_absolute() {
        config.knowledge.db_path = dir.join(&config.knowledge.db_path)
            .to_string_lossy().into_owned();
    }

    Ok(config)
}

pub fn save(config: &Config) -> anyhow::Result<()> {
    let dir = app_data_dir();
    let path = dir.join("config.toml");
    let text = toml::to_string_pretty(config)?;
    std::fs::write(&path, text)?;
    Ok(())
}

fn default_config(save_path: &str) -> Config {
    Config {
        model: ModelConfig {
            endpoint: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            context_length: 8192,
            thinking_mode: false,
            price_input: 0.0011,
            price_output: 0.0042,
        },
        save: SaveConfig {
            path: save_path.to_string(),
        },
        agent: AgentConfig {
            token_budget: 200000,
            max_steps: 10,
        },
        knowledge: KnowledgeConfig {
            db_path: "knowledge.db".into(),
        },
    }
}

/// 测试 LLM API 连通性（设置面板的"测试连接"按钮）
pub async fn test_connection(endpoint: &str, api_key: &str, model: &str) -> anyhow::Result<String> {
    if api_key.is_empty() {
        anyhow::bail!("API Key 为空，请先填写");
    }
    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "回复OK"}],
        "max_tokens": 8,
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("请求失败: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(String::from))
            .unwrap_or_else(|| text.chars().take(200).collect());
        anyhow::bail!("HTTP {}: {}", status.as_u16(), msg);
    }
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("响应解析失败: {}", e))?;
    let reply = v["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    let usage = &v["usage"];
    let tokens = usage["prompt_tokens"].as_u64().unwrap_or(0)
        + usage["completion_tokens"].as_u64().unwrap_or(0);
    Ok(format!("连接成功！模型回复「{}」（消耗 {} tok）", reply, tokens))
}
