use super::state::*;
use roxmltree::Document;
use std::path::Path;

/// 解析星露谷存档 XML → GameState
pub fn parse(path: &Path) -> anyhow::Result<GameState> {
    let xml = std::fs::read_to_string(path)?;
    parse_xml(&xml)
}

fn parse_xml(xml: &str) -> anyhow::Result<GameState> {
    let doc = Document::parse(&xml)?;
    let root = doc.root_element();

    let player = root.children()
        .find(|n| n.has_tag_name("player"))
        .ok_or_else(|| anyhow::anyhow!("存档中找不到 <player>"))?;

    let money = player.children()
        .find(|n| n.has_tag_name("money"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .unwrap_or(0);

    let day = root.children()
        .find(|n| n.has_tag_name("dayOfMonth"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<u32>().ok())
        .unwrap_or(1);

    let year = root.children()
        .find(|n| n.has_tag_name("year"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<u32>().ok())
        .unwrap_or(1);

    let season = root.children()
        .find(|n| n.has_tag_name("currentSeason"))
        .and_then(|n| n.text())
        .unwrap_or("spring")
        .trim()
        .to_string();

    let is_raining = root.children()
        .find(|n| n.has_tag_name("isRaining"))
        .and_then(|n| n.text())
        .map(|t| t.trim() == "true")
        .unwrap_or(false);

    let is_lightning = root.children()
        .find(|n| n.has_tag_name("isLightning"))
        .and_then(|n| n.text())
        .map(|t| t.trim() == "true")
        .unwrap_or(false);

    let tomorrow = root.children()
        .find(|n| n.has_tag_name("weatherForTomorrow"))
        .and_then(|n| n.text())
        .unwrap_or("Sun")
        .trim()
        .to_string();

    let daily_luck = root.children()
        .find(|n| n.has_tag_name("dailyLuck"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<f64>().ok())
        .unwrap_or(0.0);

    let skills = parse_skills(&player);

    let crops = parse_crops(&root);

    let friendships = parse_friendships(&player);

    let inventory = parse_inventory(&player);

    let chests = parse_chests(&root);

    let quests = parse_quests(&player);

    Ok(GameState {
        money,
        date: GameDate { year, season, day },
        weather: Weather { is_raining, is_lightning, tomorrow },
        daily_luck,
        skills,
        crops,
        friendships,
        inventory,
        chests,
        quests,
    })
}

fn parse_skills(player: &roxmltree::Node) -> Skills {
    let get = |name: &str| -> u32 {
        player.children()
            .find(|n| n.has_tag_name(name))
            .and_then(|n| n.text())
            .and_then(|t| t.trim().parse::<u32>().ok())
            .unwrap_or(0)
    };
    Skills {
        farming: get("farmingLevel"),
        mining: get("miningLevel"),
        combat: get("combatLevel"),
        foraging: get("foragingLevel"),
        fishing: get("fishingLevel"),
    }
}

fn parse_friendships(player: &roxmltree::Node) -> Vec<Friendship> {
    let mut result = Vec::new();

    if let Some(fs) = player.children().find(|n| n.has_tag_name("friendshipData")) {
        for item in fs.children().filter(|n| n.has_tag_name("item")) {
            let npc = item.children()
                .find(|n| n.has_tag_name("key"))
                .and_then(|k| k.children().find(|n| n.has_tag_name("string")))
                .and_then(|s| s.text())
                .unwrap_or("Unknown")
                .trim()
                .to_string();

            let points = item.children()
                .find(|n| n.has_tag_name("value"))
                .and_then(|v| v.children().find(|n| n.has_tag_name("Friendship")))
                .and_then(|f| f.children().find(|n| n.has_tag_name("Points")))
                .and_then(|p| p.text())
                .and_then(|t| t.trim().parse::<u32>().ok())
                .unwrap_or(0);

            if points > 0 {
                result.push(Friendship { npc, points });
            }
        }
    }

    result.sort_by(|a, b| b.points.cmp(&a.points));
    result
}

fn parse_quests(player: &roxmltree::Node) -> Vec<Quest> {
    let mut result = Vec::new();

    let quest_log = match player.children().find(|n| n.has_tag_name("questLog")) {
        Some(q) => q,
        None => return result,
    };

    for quest in quest_log.children().filter(|n| n.has_tag_name("Quest")) {
        let get_text = |tag: &str| -> String {
            quest.children()
                .find(|n| n.has_tag_name(tag))
                .and_then(|n| n.text())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let get_int = |tag: &str| -> i32 {
            quest.children()
                .find(|n| n.has_tag_name(tag))
                .and_then(|n| n.text())
                .and_then(|t| t.trim().parse::<i32>().ok())
                .unwrap_or(0)
        };
        let get_bool = |tag: &str| -> bool {
            quest.children()
                .find(|n| n.has_tag_name(tag))
                .and_then(|n| n.text())
                .map(|t| t.trim() == "true")
                .unwrap_or(false)
        };

        let completed = get_bool("completed");
        if completed {
            continue;
        }

        let xsi_type = quest.attributes()
            .find(|a| a.name() == "type")
            .map(|a| a.value())
            .unwrap_or("");

        let quest_type = match xsi_type {
            "ItemDeliveryQuest" => "delivery",
            "LostItemQuest" => "lost_item",
            "CollectObjective" | "ResourceCollectionQuest" => "collect",
            _ => if get_int("dailyQuest") == 1 { "daily" } else { "story" },
        }.to_string();

        let title = get_text("_questTitle");
        let description = get_text("_questDescription");
        let objective = get_text("_currentObjective");
        if title.is_empty() && objective.is_empty() {
            continue;
        }

        let target = {
            let t = get_text("target");
            if t.is_empty() { None } else { Some(t) }
        };
        let item = {
            let i = get_text("item");
            if i.is_empty() { None } else { Some(i) }
        };
        let number = {
            let n = get_int("number");
            if n > 0 { Some(n) } else { None }
        };

        result.push(Quest {
            title,
            description,
            objective,
            completed: false,
            money_reward: get_int("moneyReward"),
            days_left: get_int("daysLeft"),
            quest_type,
            target,
            item,
            number,
        });
    }

    result
}

fn parse_inventory(player: &roxmltree::Node) -> Vec<InventoryItem> {
    let mut result = Vec::new();

    if let Some(items) = player.children().find(|n| n.has_tag_name("items")) {
        for item in items.children().filter(|n| n.has_tag_name("Item") || n.is_element()) {
            let name = item.children()
                .find(|n| n.has_tag_name("name"))
                .and_then(|n| n.text())
                .unwrap_or("")
                .trim()
                .to_string();

            let stack = item.children()
                .find(|n| n.has_tag_name("stack"))
                .and_then(|n| n.text())
                .and_then(|t| t.trim().parse::<u32>().ok())
                .unwrap_or(1);

            if !name.is_empty() && name != "null" {
                result.push(InventoryItem { name, count: stack });
            }
        }
    }

    result
}

fn parse_chests(root: &roxmltree::Node) -> Vec<Chest> {
    let mut result = Vec::new();

    let locations = match root.children().find(|n| n.has_tag_name("locations")) {
        Some(l) => l,
        None => return result,
    };

    for loc in locations.children().filter(|n| n.is_element()) {
        let loc_name = loc.children()
            .find(|n| n.has_tag_name("name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        let objects = match loc.children().find(|n| n.has_tag_name("objects")) {
            Some(o) => o,
            None => continue,
        };

        for item in objects.children().filter(|n| n.has_tag_name("item")) {
            let val = match item.children().find(|n| n.has_tag_name("value")) {
                Some(v) => v,
                None => continue,
            };

            for obj in val.children().filter(|n| n.is_element()) {
                let obj_name = obj.children()
                    .find(|n| n.has_tag_name("name"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if obj_name != "Chest" {
                    continue;
                }

                let x = item.children()
                    .find(|n| n.has_tag_name("key"))
                    .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")))
                    .and_then(|v| v.children().find(|n| n.has_tag_name("X")))
                    .and_then(|x| x.text())
                    .and_then(|t| t.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);

                let y = item.children()
                    .find(|n| n.has_tag_name("key"))
                    .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")))
                    .and_then(|v| v.children().find(|n| n.has_tag_name("Y")))
                    .and_then(|y| y.text())
                    .and_then(|t| t.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);

                let mut chest_items = Vec::new();
                if let Some(items) = obj.children().find(|n| n.has_tag_name("items")) {
                    for ci in items.children().filter(|n| n.is_element()) {
                        let name = ci.children()
                            .find(|n| n.has_tag_name("name"))
                            .and_then(|n| n.text())
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        let stack = ci.children()
                            .find(|n| n.has_tag_name("stack"))
                            .and_then(|n| n.text())
                            .and_then(|t| t.trim().parse::<u32>().ok())
                            .unwrap_or(1);

                        if !name.is_empty() && name != "null" {
                            chest_items.push(InventoryItem { name, count: stack });
                        }
                    }
                }

                if !chest_items.is_empty() {
                    result.push(Chest {
                        location: loc_name.clone(),
                        x, y,
                        items: chest_items,
                    });
                }
            }
        }
    }

    result
}

fn parse_crops(root: &roxmltree::Node) -> Vec<CropStatus> {
    let mut result = Vec::new();

    let locations = match root.children().find(|n| n.has_tag_name("locations")) {
        Some(l) => l,
        None => return result,
    };

    for loc in locations.children().filter(|n| n.is_element()) {
        let name = loc.children()
            .find(|n| n.has_tag_name("name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        if name != "Farm" && name != "Greenhouse" {
            continue;
        }

        if let Some(tf) = loc.children().find(|n| n.has_tag_name("terrainFeatures")) {
            for item in tf.children().filter(|n| n.has_tag_name("item")) {
                if let Some(crop) = extract_crop_from_terrain(&item) {
                    result.push(crop);
                }
            }
        }
    }

    result.sort_by(|a, b| a.days_to_harvest.cmp(&b.days_to_harvest));
    result
}

fn extract_crop_from_terrain(item: &roxmltree::Node) -> Option<CropStatus> {
    let value = item.children().find(|n| n.has_tag_name("value"))?;
    let terrain = value.children().find(|n| n.has_tag_name("TerrainFeature"))?;

    let xsi_type = terrain.attributes()
        .find(|a| a.name() == "type")
        .map(|a| a.value());

    if xsi_type.is_none() {
        return None;
    }

    let crop = terrain.children().find(|n| n.has_tag_name("crop"))?;
    let idx = crop.children()
        .find(|n| n.has_tag_name("indexOfHarvest"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())?;

    let x = item.children()
        .find(|n| n.has_tag_name("key"))
        .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")))
        .and_then(|v| v.children().find(|n| n.has_tag_name("X")))
        .and_then(|x| x.text())
        .and_then(|t| t.trim().parse::<f32>().ok())
        .unwrap_or(0.0);

    let y = item.children()
        .find(|n| n.has_tag_name("key"))
        .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")))
        .and_then(|v| v.children().find(|n| n.has_tag_name("Y")))
        .and_then(|y| y.text())
        .and_then(|t| t.trim().parse::<f32>().ok())
        .unwrap_or(0.0);

    let current_phase = crop.children()
        .find(|n| n.has_tag_name("currentPhase"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .unwrap_or(0);

    let day_of_phase = crop.children()
        .find(|n| n.has_tag_name("dayOfCurrentPhase"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .unwrap_or(0);

    let is_dead = crop.children()
        .find(|n| n.has_tag_name("dead"))
        .and_then(|n| n.text())
        .map(|t| t.trim() == "true")
        .unwrap_or(false);

    let full_grown = crop.children()
        .find(|n| n.has_tag_name("fullGrown"))
        .and_then(|n| n.text())
        .map(|t| t.trim() == "true")
        .unwrap_or(false);

    let phase_days: Vec<i32> = crop.children()
        .find(|n| n.has_tag_name("phaseDays"))
        .map(|pd| pd.children()
            .filter(|n| n.has_tag_name("int"))
            .filter_map(|n| n.text())
            .filter_map(|t| t.trim().parse::<i32>().ok())
            .collect())
        .unwrap_or_default();

    let total_phases = phase_days.len() as i32;

    let days_to_harvest = if is_dead {
        -1
    } else if full_grown || current_phase >= total_phases.saturating_sub(1) {
        0
    } else {
        let remaining = phase_days.get(current_phase as usize)
            .map(|d| d - day_of_phase)
            .unwrap_or(0);
        // 只累加到倒数第二阶段，排除最后的 99999（成熟阶段）
        let future: i32 = phase_days
            .get((current_phase + 1) as usize..(total_phases - 1).max(1) as usize)
            .map(|slice| slice.iter().sum())
            .unwrap_or(0);
        remaining + future
    };

    Some(CropStatus {
        item_id: idx,
        name: crop_id_to_name(idx),
        x, y,
        current_phase,
        total_phases,
        days_to_harvest,
        is_dead,
    })
}

fn crop_id_to_name(id: i32) -> String {
    match id {
        24 => "防风草".into(),
        188 => "花椰菜".into(),
        190 => "马铃薯".into(),
        192 => "花椰菜".into(),
        252 => "大黄".into(),
        254 => "甜瓜".into(),
        256 => "番茄".into(),
        257 => "辣椒".into(),
        258 => "蓝莓".into(),
        260 => "啤酒花".into(),
        262 => "小麦".into(),
        264 => "萝卜".into(),
        266 => "茄子".into(),
        270 => "南瓜".into(),
        272 => "玉米".into(),
        276 => "茄子".into(),
        280 => "山药".into(),
        281 => "甜瓜".into(),
        282 => "蔓越莓".into(),
        283 => "向日葵".into(),
        284 => "红叶卷心菜".into(),
        300 => "苋菜".into(),
        304 => "辣椒".into(),
        376 => "罂粟".into(),
        396 => "野种".into(),
        416 => "蓝莓".into(),
        417 => "玉米".into(),
        418 => "茄子".into(),
        421 => "啤酒花".into(),
        433 => "咖啡豆".into(),
        591 => "甜宝石浆果".into(),
        593 => "茶叶".into(),
        833 => "姜".into(),
        834 => "茶叶".into(),
        _ => format!("未知作物#{}", id),
    }
}
