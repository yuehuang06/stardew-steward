use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct KnowledgeBase {
    conn: Connection,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CropInfo {
    pub season: String,
    pub growth_days: i32,
    #[serde(default)]
    pub regrows: bool,
    #[serde(default)]
    pub sell_price: i32,
    #[serde(default)]
    pub seed_cost: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NpcInfo {
    pub birthday: String,
    #[serde(default)]
    pub loves: Vec<String>,
    #[serde(default)]
    pub likes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FishInfo {
    pub season: String,
    #[serde(default)]
    pub weather: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub location: String,
}

impl KnowledgeBase {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let conn = if std::path::Path::new(path).exists() {
            Connection::open(path)?
        } else {
            let conn = Connection::open(path)?;
            Self::init_tables(&conn)?;
            conn
        };

        let kb = Self { conn };
        kb.load_seed_data_if_empty()?;
        Ok(kb)
    }

    fn init_tables(conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS crops (
                name TEXT PRIMARY KEY,
                season TEXT,
                growth_days INTEGER,
                regrows INTEGER,
                sell_price INTEGER,
                seed_cost INTEGER
            );
            CREATE TABLE IF NOT EXISTS npcs (
                name TEXT PRIMARY KEY,
                birthday TEXT,
                loves TEXT,
                likes TEXT
            );
            CREATE TABLE IF NOT EXISTS fish (
                name TEXT PRIMARY KEY,
                season TEXT,
                weather TEXT,
                time TEXT,
                location TEXT
            );"
        )?;
        Ok(())
    }

    fn load_seed_data_if_empty(&self) -> anyhow::Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM crops", [], |row| row.get(0)
        )?;
        if count > 0 {
            return Ok(());
        }

        let crops_text = std::fs::read_to_string("data/crops.json")
            .unwrap_or_else(|_| include_str!("../../data/crops.json").to_string());
        let npcs_text = std::fs::read_to_string("data/npcs.json")
            .unwrap_or_else(|_| include_str!("../../data/npcs.json").to_string());
        let fish_text = std::fs::read_to_string("data/fish.json")
            .unwrap_or_else(|_| include_str!("../../data/fish.json").to_string());

        self.import_crops(&crops_text)?;
        self.import_npcs(&npcs_text)?;
        self.import_fish(&fish_text)?;

        Ok(())
    }

    fn import_crops(&self, json: &str) -> anyhow::Result<()> {
        let map: HashMap<String, CropInfo> = serde_json::from_str(json)?;
        for (name, c) in map {
            self.conn.execute(
                "INSERT OR REPLACE INTO crops VALUES (?, ?, ?, ?, ?, ?)",
                params![name, c.season, c.growth_days, c.regrows as i32, c.sell_price, c.seed_cost],
            )?;
        }
        Ok(())
    }

    fn import_npcs(&self, json: &str) -> anyhow::Result<()> {
        let map: HashMap<String, NpcInfo> = serde_json::from_str(json)?;
        for (name, n) in map {
            self.conn.execute(
                "INSERT OR REPLACE INTO npcs VALUES (?, ?, ?, ?)",
                params![name, n.birthday, n.loves.join(", "), n.likes.join(", ")],
            )?;
        }
        Ok(())
    }

    fn import_fish(&self, json: &str) -> anyhow::Result<()> {
        let map: HashMap<String, FishInfo> = serde_json::from_str(json)?;
        for (name, f) in map {
            self.conn.execute(
                "INSERT OR REPLACE INTO fish VALUES (?, ?, ?, ?, ?)",
                params![name, f.season, f.weather, f.time, f.location],
            )?;
        }
        Ok(())
    }

    pub fn search(&self, keyword: &str) -> anyhow::Result<String> {
        let keywords: Vec<String> = keyword.split_whitespace()
            .map(|k| format!("%{}%", k))
            .collect();
        let mut results = Vec::new();

        // 对每个关键词查 crops，用 OR 拼接
        if !keywords.is_empty() {
            let crop_clauses: Vec<String> = keywords.iter()
                .map(|_| "name LIKE ?".to_string())
                .collect();
            let sql = format!(
                "SELECT name, season, growth_days, regrows, sell_price, seed_cost FROM crops WHERE {}",
                crop_clauses.join(" OR ")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::ToSql> = keywords.iter()
                .map(|k| k as &dyn rusqlite::ToSql)
                .collect();
            let rows = stmt.query_map(params.as_slice(), |row| {
                Ok(format!(
                    "作物|{}: 季节={}, 生长{}天, {}复收, 售价{}g, 种子{}g",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    if row.get::<_, i64>(3)? != 0 { "可" } else { "不" },
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })?;
            for row in rows {
                if let Ok(text) = row { results.push(text); }
            }

            // npcs
            let npc_clauses: Vec<String> = keywords.iter()
                .map(|_| "name LIKE ?".to_string())
                .collect();
            let sql = format!(
                "SELECT name, birthday, loves, likes FROM npcs WHERE {}",
                npc_clauses.join(" OR ")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::ToSql> = keywords.iter()
                .map(|k| k as &dyn rusqlite::ToSql)
                .collect();
            let rows = stmt.query_map(params.as_slice(), |row| {
                Ok(format!(
                    "NPC|{}: 生日={}, 最爱[{}], 喜欢[{}]",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;
            for row in rows {
                if let Ok(text) = row { results.push(text); }
            }

            // fish
            let fish_clauses: Vec<String> = keywords.iter()
                .map(|_| "name LIKE ?".to_string())
                .collect();
            let sql = format!(
                "SELECT name, season, weather, time, location FROM fish WHERE {}",
                fish_clauses.join(" OR ")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::ToSql> = keywords.iter()
                .map(|k| k as &dyn rusqlite::ToSql)
                .collect();
            let rows = stmt.query_map(params.as_slice(), |row| {
                Ok(format!(
                    "鱼|{}: 季节={}, 天气={}, 时间={}, 地点={}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?;
            for row in rows {
                if let Ok(text) = row { results.push(text); }
            }
        }

        if results.is_empty() {
            Ok(format!("未找到与「{}」相关的知识。", keyword))
        } else {
            Ok(results.join("\n"))
        }
    }

    /// 精确查询作物售价（供求解器用）
    pub fn get_crop_price(&self, name: &str) -> Option<i32> {
        self.conn
            .query_row("SELECT sell_price FROM crops WHERE name = ?", [name], |row| row.get(0))
            .ok()
    }

    /// 精确查询 NPC 生日（供求解器用）
    pub fn get_npc_birthday(&self, name: &str) -> Option<String> {
        self.conn
            .query_row("SELECT birthday FROM npcs WHERE name = ?", [name], |row| row.get(0))
            .ok()
    }
}
