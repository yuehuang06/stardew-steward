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


    let friendships = parse_friendships(&player);

    let inventory = parse_inventory(&player);

    let chests = parse_chests(&root);

    let quests = parse_quests(&player);

    let (buildings, junimo_huts) = parse_buildings(&root);

    let (crops, sprinklers, fruit_trees, trees) = parse_crops_and_trees(&root);

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
        buildings,
        junimo_huts,
        sprinklers,
        fruit_trees,
        trees,
    })
}

/// 建筑类型 → 中文名（含功能提示）
pub fn building_type_cn(kind: &str) -> String {
    match kind {
        "Barn" => "畜棚".into(),
        "Big Barn" => "大畜棚".into(),
        "Deluxe Barn" => "高级畜棚".into(),
        "Coop" => "鸡舍".into(),
        "Big Coop" => "大鸡舍".into(),
        "Deluxe Coop" => "高级鸡舍".into(),
        "Silo" => "筒仓（储存干草）".into(),
        "Shed" => "小屋（储物）".into(),
        "Big Shed" => "大棚（储物）".into(),
        "Mill" => "磨坊（产面粉/糖）".into(),
        "Junimo Hut" => "Junimo小屋（自动浇水+收获）".into(),
        "Stable" => "马厩".into(),
        "Greenhouse" => "温室".into(),
        "Slime Hutch" => "史莱姆屋".into(),
        "Farmhouse" => "农舍".into(),
        "Cabin" => "小木屋".into(),
        "Well" => "水井".into(),
        "Shipping Bin" => "出货箱".into(),
        "Gold Clock" | "Island Obelisk" | "Desert Obelisk" | "Water Obelisk" | "Earth Obelisk" => {
            format!("{}（图腾柱）", kind.replace(" Obelisk", "传送塔"))
        }
        other => other.to_string(),
    }
}

fn parse_buildings(root: &roxmltree::Node) -> (Vec<BuildingInfo>, u32) {
    let mut map: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    let mut junimo_huts = 0u32;

    // 建筑在 Farm location 的 <buildings> 下
    if let Some(locations) = root.children().find(|n| n.has_tag_name("locations")) {
        for loc in locations.children().filter(|n| n.is_element()) {
            let name = loc.children()
                .find(|n| n.has_tag_name("name"))
                .and_then(|n| n.text())
                .unwrap_or("")
                .trim()
                .to_string();
            if name != "Farm" {
                continue;
            }
            if let Some(bldgs) = loc.children().find(|n| n.has_tag_name("buildings")) {
                for b in bldgs.children().filter(|n| n.has_tag_name("Building")) {
                    let kind = b.children()
                        .find(|n| n.has_tag_name("buildingType"))
                        .and_then(|n| n.text())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if kind.is_empty() {
                        continue;
                    }
                    *map.entry(kind.clone()).or_insert(0) += 1;
                    if kind == "Junimo Hut" {
                        junimo_huts += 1;
                    }
                }
            }
        }
    }

    let buildings = map.into_iter()
        .map(|(kind, count)| BuildingInfo {
            name: building_type_cn(&kind),
            kind,
            count,
        })
        .collect();
    (buildings, junimo_huts)
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

        // 普通地点（Farm/Greenhouse/矿洞等）里的木箱
        if let Some(objects) = loc.children().find(|n| n.has_tag_name("objects")) {
            extract_chests_from_objects(&objects, &loc_name, &mut result);
        }

        // 建筑内部（小屋/鸡舍/畜棚的 indoors）的木箱
        if loc_name == "Farm" {
            if let Some(bldgs) = loc.children().find(|n| n.has_tag_name("buildings")) {
                for b in bldgs.children().filter(|n| n.has_tag_name("Building")) {
                    let kind = b.children()
                        .find(|n| n.has_tag_name("buildingType"))
                        .and_then(|n| n.text())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let label = building_type_cn(&kind);
                    if let Some(indoors) = b.children().find(|n| n.has_tag_name("indoors")) {
                        for room in indoors.children().filter(|n| n.is_element()) {
                            if let Some(objects) = room.children().find(|n| n.has_tag_name("objects")) {
                                extract_chests_from_objects(&objects, &label, &mut result);
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn extract_chests_from_objects(objects: &roxmltree::Node, loc_label: &str, result: &mut Vec<Chest>) {
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
                    location: loc_label.to_string(),
                    x, y,
                    items: chest_items,
                });
            }
        }
    }
}

/// 计算一个地点内所有洒水器的覆盖格集合
/// - Sprinkler(基础): 十字4格; +增压喷嘴 → 3x3
/// - Quality Sprinkler: 3x3; +增压喷嘴 → 5x5
/// - Iridium Sprinkler: 5x5; +增压喷嘴 → 7x7
fn sprinkler_coverage(loc: &roxmltree::Node) -> std::collections::HashSet<(i32, i32)> {
    let mut covered = std::collections::HashSet::new();
    let objects = match loc.children().find(|n| n.has_tag_name("objects")) {
        Some(o) => o,
        None => return covered,
    };
    for item in objects.children().filter(|n| n.has_tag_name("item")) {
        let val = match item.children().find(|n| n.has_tag_name("value")) {
            Some(v) => v,
            None => continue,
        };
        let obj = match val.children().find(|n| n.is_element()) {
            Some(o) => o,
            None => continue,
        };
        let name = obj.children()
            .find(|n| n.has_tag_name("name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();
        let radius = match name.as_str() {
            "Sprinkler" => 1,            // 十字(特判) / 喷嘴后 3x3
            "Quality Sprinkler" => 2,    // 3x3 / 喷嘴后 5x5
            "Iridium Sprinkler" => 3,    // 5x5 / 喷嘴后 7x7
            _ => continue,
        };
        let has_nozzle = obj.children()
            .find(|n| n.has_tag_name("heldObject"))
            .map(|h| {
                h.children()
                    .filter(|n| n.has_tag_name("name"))
                    .filter_map(|n| n.text())
                    .any(|t| t.contains("Pressure Nozzle"))
            })
            .unwrap_or(false);

        let pos = item.children()
            .find(|n| n.has_tag_name("key"))
            .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")));
        let (x, y) = match pos {
            Some(v) => {
                let gx = v.children().find(|n| n.has_tag_name("X"))
                    .and_then(|n| n.text()).and_then(|t| t.trim().parse::<i32>().ok()).unwrap_or(-999);
                let gy = v.children().find(|n| n.has_tag_name("Y"))
                    .and_then(|n| n.text()).and_then(|t| t.trim().parse::<i32>().ok()).unwrap_or(-999);
                (gx, gy)
            }
            None => continue,
        };

        let r = if has_nozzle { radius + 1 } else { radius };
        // 基础洒水器无喷嘴: 只浇十字4格
        if name == "Sprinkler" && !has_nozzle {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                covered.insert((x + dx, y + dy));
            }
        } else {
            for dx in -r..=r {
                for dy in -r..=r {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    covered.insert((x + dx, y + dy));
                }
            }
        }
    }
    covered
}

fn parse_crops_and_trees(
    root: &roxmltree::Node,
) -> (Vec<CropStatus>, u32, Vec<FruitTreeStatus>, Vec<TreeStatus>) {
    let mut crops = Vec::new();
    let mut sprinkler_count = 0u32;
    let mut fruit_map: std::collections::BTreeMap<String, (u32, u32, u32)> = Default::default();
    let mut tree_map: std::collections::BTreeMap<String, (u32, u32)> = Default::default();

    let locations = match root.children().find(|n| n.has_tag_name("locations")) {
        Some(l) => l,
        None => return (crops, 0, Vec::new(), Vec::new()),
    };

    for loc in locations.children().filter(|n| n.is_element()) {
        let name = loc.children()
            .find(|n| n.has_tag_name("name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        if name != "Farm" && name != "Greenhouse" && name != "IslandWest" {
            continue;
        }

        let coverage = sprinkler_coverage(&loc);
        // 精确计数洒水器对象数
        if let Some(objects) = loc.children().find(|n| n.has_tag_name("objects")) {
            for item in objects.children().filter(|n| n.has_tag_name("item")) {
                if let Some(v) = item.children().find(|n| n.has_tag_name("value")) {
                    if let Some(o) = v.children().find(|n| n.is_element()) {
                        if let Some(nm) = o.children().find(|n| n.has_tag_name("name")).and_then(|n| n.text()) {
                            if nm.trim().ends_with("Sprinkler") {
                                sprinkler_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if let Some(tf) = loc.children().find(|n| n.has_tag_name("terrainFeatures")) {
            for item in tf.children().filter(|n| n.has_tag_name("item")) {
                let value = match item.children().find(|n| n.has_tag_name("value")) {
                    Some(v) => v,
                    None => continue,
                };
                let terrain = match value.children().find(|n| n.has_tag_name("TerrainFeature")) {
                    Some(t) => t,
                    None => continue,
                };
                let xsi = terrain.attributes()
                    .find(|a| a.name() == "type")
                    .map(|a| a.value())
                    .unwrap_or("");

                let tile = item.children()
                    .find(|n| n.has_tag_name("key"))
                    .and_then(|k| k.children().find(|n| n.has_tag_name("Vector2")))
                    .map(|v| {
                        (
                            v.children().find(|n| n.has_tag_name("X"))
                                .and_then(|n| n.text()).and_then(|t| t.trim().parse::<i32>().ok()).unwrap_or(-999),
                            v.children().find(|n| n.has_tag_name("Y"))
                                .and_then(|n| n.text()).and_then(|t| t.trim().parse::<i32>().ok()).unwrap_or(-999),
                        )
                    });

                match xsi {
                    "HoeDirt" => {
                        if let Some(mut crop) = extract_crop_from_terrain(&item) {
                            if let Some((tx, ty)) = tile {
                                crop.sprinkler_covered = coverage.contains(&(tx, ty));
                            }
                            crops.push(crop);
                        }
                    }
                    "FruitTree" => {
                        let (ft_name, mature, ready) = parse_fruit_tree(&terrain);
                        let e = fruit_map.entry(ft_name).or_insert((0, 0, 0));
                        e.0 += 1;
                        if mature { e.1 += 1; }
                        if ready { e.2 += 1; }
                    }
                    "Tree" => {
                        let (t_name, mature) = parse_tree(&terrain);
                        let e = tree_map.entry(t_name).or_insert((0, 0));
                        e.0 += 1;
                        if mature { e.1 += 1; }
                    }
                    _ => {}
                }
            }
        }
    }

    crops.sort_by(|a, b| a.days_to_harvest.cmp(&b.days_to_harvest));

    let fruit_trees = fruit_map.into_iter()
        .map(|(name, (count, mature, ready))| FruitTreeStatus { name, count, mature, ready })
        .collect();
    let trees = tree_map.into_iter()
        .map(|(name, (count, mature))| TreeStatus { name, count, mature })
        .collect();

    (crops, sprinkler_count, fruit_trees, trees)
}

/// 果树: 优先取 <fruit><name>，回退 treeId 映射
fn parse_fruit_tree(terrain: &roxmltree::Node) -> (String, bool, bool) {
    let name = terrain.children()
        .find(|n| n.has_tag_name("fruit"))
        .and_then(|f| f.children().find(|n| n.has_tag_name("name")))
        .and_then(|n| n.text())
        .map(|t| fruit_name_cn(t.trim()))
        .or_else(|| {
            terrain.children()
                .find(|n| n.has_tag_name("treeId"))
                .and_then(|n| n.text())
                .and_then(|t| t.trim().parse::<i32>().ok())
                .map(tree_id_cn)
        })
        .unwrap_or_else(|| "未知果树".into());

    // daysUntilMature <= 0 表示已成熟（成熟后为负数）
    let mature = terrain.children()
        .find(|n| n.has_tag_name("daysUntilMature"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .map(|d| d <= 0)
        .unwrap_or(false);

    // fruitsOnTree > 0 = 当前有果子可摇（可能为 xsi:nil）
    let ready = terrain.children()
        .find(|n| n.has_tag_name("fruitsOnTree"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .map(|f| f > 0)
        .unwrap_or(false);

    (name, mature, ready)
}

fn fruit_name_cn(en: &str) -> String {
    match en {
        "Cherry" => "樱桃树",
        "Apricot" => "杏树",
        "Orange" => "橙树",
        "Peach" => "桃树",
        "Pomegranate" => "石榴树",
        "Apple" => "苹果树",
        "Mango" => "芒果树",
        "Banana" => "香蕉树",
        "Coconut" => "棕榈树",
        _ => return format!("{}树", en),
    }.to_string()
}

fn tree_id_cn(id: i32) -> String {
    match id {
        628 => "樱桃树",
        629 => "杏树",
        630 => "橙树",
        631 => "桃树",
        632 => "石榴树",
        633 => "苹果树",
        634 => "芒果树",
        635 => "香蕉树",
        _ => "果树",
    }.to_string()
}

fn parse_tree(terrain: &roxmltree::Node) -> (String, bool) {
    let ttype = terrain.children()
        .find(|n| n.has_tag_name("treeType"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .unwrap_or(-1);
    let name = match ttype {
        1 => "枫树",
        2 => "橡树",
        3 => "松树",
        6 => "棕榈树",
        7 => "桃花心木",
        8 => "蘑菇树",
        9 => "绿茶树",
        13 => "苔藓树",
        _ => "野生树苗",
    }.to_string();
    let mature = terrain.children()
        .find(|n| n.has_tag_name("growthStage"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i32>().ok())
        .map(|g| g >= 4)
        .unwrap_or(false);
    (name, mature)
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

    // 浇水状态: HoeDirt 的 state 字段 bit 0
    let watered = terrain.children()
        .find(|n| n.has_tag_name("state"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<u32>().ok())
        .map(|s| s & 1 == 1)
        .unwrap_or(false);

    let crop = terrain.children().find(|n| n.has_tag_name("crop"))?;

    // 1.6 野生种子作物: forageCrop=true，无 indexOfHarvest
    let is_forage = crop.children()
        .find(|n| n.has_tag_name("forageCrop"))
        .and_then(|n| n.text())
        .map(|t| t.trim() == "true")
        .unwrap_or(false);

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
    // 99999 收获槽位（最后一个 phaseDays 条目）
    let last_phase = total_phases.saturating_sub(1);
    let in_harvest_phase = current_phase >= last_phase;

    // 1.6 收获规则（经实测存档验证）:
    // - fullGrown=true 表示已成熟过/已收获过
    // - 收获后 dayOfCurrentPhase 重置为再生天数，每夜 -1，减到负数后可再收获
    // - 从未收获的成熟作物: fullGrown=true 且 dop<0（首次成熟时置为负值）
    // - fullGrown=false 且在收获槽位 = 尚未真正成熟（dop 计数中）
    let harvestable = !is_dead && in_harvest_phase && full_grown && day_of_phase < 0;

    let days_to_harvest = if is_dead {
        -1
    } else if harvestable {
        0
    } else if in_harvest_phase {
        if full_grown {
            // 再生倒计时: dop 每夜 -1，到 -1 后可收获 → 还需 dop+1 晚
            day_of_phase + 1
        } else {
            // 未成熟但已进入末位阶段（少见状态），按至少 1 天估
            1
        }
    } else {
        let remaining = phase_days.get(current_phase as usize)
            .map(|d| d - day_of_phase)
            .unwrap_or(0);
        // 只累加到收获槽位之前，排除最后的 99999
        let future: i32 = phase_days
            .get((current_phase + 1) as usize..last_phase.max(1) as usize)
            .map(|slice| slice.iter().sum())
            .unwrap_or(0);
        remaining + future
    };

    let (item_id, name) = if is_forage {
        (-1, "野生作物（野种）".to_string())
    } else {
        let idx = crop.children()
            .find(|n| n.has_tag_name("indexOfHarvest"))
            .and_then(|n| n.text())
            .and_then(|t| t.trim().parse::<i32>().ok())?;
        (idx, crop_id_to_name(idx))
    };

    Some(CropStatus {
        item_id,
        name,
        x, y,
        current_phase,
        total_phases,
        days_to_harvest,
        is_dead,
        watered,
        harvestable,
        sprinkler_covered: false,
    })
}

/// 作物 ID → 中文名
/// 映射经 SDV 1.6.15 实际存档验证（seedIndex + phaseDays + 官方 Data/Crops 交叉对照）
pub fn crop_id_to_name(id: i32) -> String {
    match id {
        // 春季
        24 => "防风草".into(),       // seed 472
        188 => "青豆".into(),        // seed 473
        190 => "花椰菜".into(),      // seed 474
        192 => "马铃薯".into(),      // seed 475
        242 => "大蒜".into(),
        248 => "甘蓝".into(),        // seed 476
        250 => "蓝爵士".into(),      // seed 477
        252 => "大黄".into(),        // seed 478, 13天
        271 => "未碾稻米".into(),    // seed 273 (Rice Shoot)
        // 夏季
        254 => "甜瓜".into(),
        256 => "番茄".into(),
        258 => "蓝莓".into(),
        260 => "辣椒".into(),        // seed 482, 5天+3再生
        262 => "小麦".into(),
        264 => "萝卜".into(),
        266 => "红叶卷心菜".into(),
        268 => "杨桃".into(),        // seed 486, 13天
        304 => "啤酒花".into(),      // 官方经典 ID
        376 => "罂粟".into(),
        400 => "草莓".into(),        // seed 745
        593 => "夏季闪光花".into(),
        834 => "西葫芦".into(),      // 1.6
        // 秋季
        270 => "玉米".into(),        // seed 487, 14天+4再生
        272 => "茄子".into(),        // seed 488, 5天+5再生
        274 => "洋蓟".into(),        // seed 489, 8天
        276 => "南瓜".into(),        // seed 490, 13天
        278 => "小白菜".into(),
        280 => "山药".into(),        // seed 492, 10天
        282 => "蔓越莓".into(),      // seed 493, 7天+5再生
        284 => "甜菜".into(),
        299 | 300 => "苋菜".into(),
        398 => "葡萄".into(),        // seed 301 (Grape Starter), 10天
        421 => "向日葵".into(),      // seed 431, 8天
        595 => "玫瑰仙子".into(),    // seed 425, 12天
        // 冬季
        597 => "霜瓜".into(),        // 1.6 Powdermelon
        // 特殊
        433 => "咖啡豆".into(),
        454 => "远古水果".into(),    // 28天+7再生
        591 => "甜宝石浆果".into(),
        815 => "茶叶".into(),
        830 => "芋头".into(),        // 1.6 Taro
        833 => "胡萝卜".into(),      // 1.6 Carrot
        _ => format!("未知作物#{}", id),
    }
}
