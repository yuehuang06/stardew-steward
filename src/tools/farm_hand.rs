use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::knowledge::KnowledgeBase;
use crate::parser::save::crop_id_to_name;
use serde::Serialize;

#[derive(Serialize)]
struct FarmResult {
    action: String,
    affected: u32,
    money_before: i32,
    money_after: i32,
    crop_details: Vec<String>,
    backup_path: String,
}

/// farm_hand: 托管农场操作
/// - water_all: 标记所有作物为已浇水
/// - harvest_all: 收获成熟作物并自动出售（加钱）
/// - clear_dead: 清理枯死的作物
/// - rollback_day: 回档到前一天（用游戏自带的 _old 存档覆盖当前存档）
pub fn execute(path: &str, action: &str, kb: &Arc<Mutex<KnowledgeBase>>) -> anyhow::Result<String> {
    let save_path = Path::new(path);

    if !save_path.exists() {
        return Err(anyhow::anyhow!("存档文件不存在: {}", path));
    }

    // 检测星露谷物语是否正在运行
    if is_game_running() {
        return Err(anyhow::anyhow!(
            "检测到星露谷物语正在运行！请先关闭游戏再执行农场操作，否则修改会被游戏覆盖。关闭后重试。"
        ));
    }

    if action == "rollback_day" {
        return rollback_day(path);
    }

    let xml = std::fs::read_to_string(save_path)
        .map_err(|e| anyhow::anyhow!("读取存档失败: {}（游戏是否仍在运行？请先关闭游戏）", e))?;

    let backup_path = format!("{}.bak", path);
    std::fs::write(&backup_path, &xml)
        .map_err(|e| anyhow::anyhow!("备份存档失败: {}", e))?;

    let money_before = extract_money(&xml);

    let (modified, affected, details, money_after) = match action {
        "water_all" => water_all(&xml, money_before),
        "harvest_all" => {
            let kb = kb.lock()
                .map_err(|e| anyhow::anyhow!("知识库锁异常: {}", e))?;
            harvest_all(&xml, money_before, &kb)
        }
        "clear_dead" => clear_dead(&xml, money_before),
        _ => return Err(anyhow::anyhow!(
            "未知操作: {}。支持: water_all, harvest_all, clear_dead, rollback_day", action
        )),
    };

    let final_xml = if money_after != money_before {
        replace_first_money(&modified, money_after)
    } else {
        modified
    };

    if roxmltree::Document::parse(&final_xml).is_err() {
        if std::fs::write(save_path, &xml).is_err() {
            return Err(anyhow::anyhow!(
                "修改后 XML 校验失败，且自动回滚也失败了！请手动从备份恢复: {}", backup_path
            ));
        }
        return Err(anyhow::anyhow!("修改后 XML 校验失败，已自动回滚到原始存档"));
    }

    std::fs::write(save_path, &final_xml)
        .map_err(|e| anyhow::anyhow!(
            "写入存档失败: {}。原始存档备份在: {}", e, backup_path
        ))?;

    let result = FarmResult {
        action: action.to_string(),
        affected,
        money_before,
        money_after,
        crop_details: details,
        backup_path,
    };
    Ok(serde_json::to_string_pretty(&result)?)
}

fn rollback_day(path: &str) -> anyhow::Result<String> {
    let old_path = format!("{}_old", path);
    if !Path::new(&old_path).exists() {
        return Err(anyhow::anyhow!("未找到前一天的存档: {}", old_path));
    }

    let current_xml = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("读取当前存档失败: {}", e))?;
    let old_xml = std::fs::read_to_string(&old_path)
        .map_err(|e| anyhow::anyhow!("读取前一天的存档失败: {}", e))?;

    if roxmltree::Document::parse(&old_xml).is_err() {
        return Err(anyhow::anyhow!("前一天的存档 XML 校验失败，已取消回档"));
    }

    let backup_path = format!("{}.bak", path);
    std::fs::write(&backup_path, &current_xml)
        .map_err(|e| anyhow::anyhow!("备份当前存档失败: {}", e))?;

    let money_before = extract_money(&current_xml);
    let money_after = extract_money(&old_xml);
    let day_before = extract_int(&current_xml, "dayOfMonth").unwrap_or(0);
    let day_after = extract_int(&old_xml, "dayOfMonth").unwrap_or(0);

    std::fs::write(path, &old_xml)
        .map_err(|e| anyhow::anyhow!(
            "写入存档失败: {}。当前存档备份在: {}", e, backup_path
        ))?;

    let result = FarmResult {
        action: "rollback_day".to_string(),
        affected: 1,
        money_before,
        money_after,
        crop_details: vec![
            format!("回档前: 第{}天, 资金{}g", day_before, money_before),
            format!("回档后: 第{}天, 资金{}g", day_after, money_after),
            format!("当天存档已备份到: {}", backup_path),
        ],
        backup_path,
    };
    Ok(serde_json::to_string_pretty(&result)?)
}

const HOE_DIRT_OPEN: &str = "<TerrainFeature xsi:type=\"HoeDirt\">";
const HOE_DIRT_CLOSE: &str = "</TerrainFeature>";

fn is_game_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq StardewValley.exe", "/FO", "CSV", "/NH"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("StardewValley.exe");
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = std::process::Command::new("pgrep")
            .args(["-i", "stardew"])
            .output()
        {
            return !output.stdout.is_empty();
        }
        false
    }
}

fn extract_money(xml: &str) -> i32 {
    let m = "<money>";
    let c = "</money>";
    if let Some(p) = xml.find(m) {
        let after = &xml[p + m.len()..];
        if let Some(e) = after.find(c) {
            return after[..e].trim().parse().unwrap_or(0);
        }
    }
    0
}

fn replace_first_money(xml: &str, new_money: i32) -> String {
    let m = "<money>";
    let c = "</money>";
    if let Some(p) = xml.find(m) {
        let after = &xml[p + m.len()..];
        if let Some(e) = after.find(c) {
            return format!("{}{}{}", &xml[..p + m.len()], new_money, &after[e..]);
        }
    }
    xml.to_string()
}

fn split_blocks(xml: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut pos = 0;
    while let Some(start) = xml[pos..].find(HOE_DIRT_OPEN) {
        let abs = pos + start;
        if abs > pos {
            segments.push(xml[pos..abs].to_string());
        }
        let rest = &xml[abs..];
        if let Some(rel_end) = rest.find(HOE_DIRT_CLOSE) {
            let abs_end = abs + rel_end + HOE_DIRT_CLOSE.len();
            segments.push(xml[abs..abs_end].to_string());
            pos = abs_end;
        } else {
            break;
        }
    }
    if pos < xml.len() {
        segments.push(xml[pos..].to_string());
    }
    segments
}

fn water_all(xml: &str, money: i32) -> (String, u32, Vec<String>, i32) {
    let segments = split_blocks(xml);
    let mut result = String::new();
    let mut count = 0u32;

    for seg in &segments {
        if seg.starts_with(HOE_DIRT_OPEN) {
            let modified = replace_state_watered(seg);
            if modified != *seg {
                count += 1;
            }
            result.push_str(&modified);
        } else {
            result.push_str(seg);
        }
    }
    (result, count, vec![format!("已浇水 {} 块耕地", count)], money)
}

fn replace_state_watered(block: &str) -> String {
    let marker = "<state>";
    let close = "</state>";
    let mut result = String::new();
    let mut rest = block;
    while let Some(p) = rest.find(marker) {
        result.push_str(&rest[..p + marker.len()]);
        let after = &rest[p + marker.len()..];
        if let Some(e) = after.find(close) {
            let val: u32 = after[..e].trim().parse().unwrap_or(0);
            result.push_str(&(val | 1).to_string());
            result.push_str(close);
            rest = &after[e + close.len()..];
        } else {
            result.push_str(after);
            break;
        }
    }
    result.push_str(rest);
    result
}

fn harvest_all(
    xml: &str,
    money: i32,
    kb: &KnowledgeBase,
) -> (String, u32, Vec<String>, i32) {
    let mut segments = split_blocks(xml);
    let mut total_income = 0i32;
    let mut total_count = 0u32;
    let mut by_name: std::collections::HashMap<String, (u32, i32)> = std::collections::HashMap::new();

    for seg in segments.iter_mut() {
        if !seg.starts_with(HOE_DIRT_OPEN) {
            continue;
        }
        // 1.6 收获规则: fullGrown=true 且 dayOfCurrentPhase < 0 才是真正可收获
        let fg = seg.contains("<fullGrown>true</fullGrown>");
        let dop = extract_int(seg, "dayOfCurrentPhase").unwrap_or(i32::MAX);
        if !fg || dop >= 0 || seg.contains("<dead>true</dead>") {
            continue;
        }
        let Some(crop_id) = extract_int(seg, "indexOfHarvest") else { continue };
        let name = crop_id_to_name(crop_id);
        let price = kb.get_crop_price(&name).unwrap_or(0);
        let info = kb.get_crop_info(&name);

        total_income += price;
        total_count += 1;
        let e = by_name.entry(name.clone()).or_insert((0, price));
        e.0 += 1;

        let block = seg.clone();
        let mut modified = block.clone();
        if info.as_ref().map(|c| c.regrows).unwrap_or(false) {
            // 1.6 再生: 保留在收获槽位(currentPhase 不动)，fullGrown 保持 true，
            // dayOfCurrentPhase 重置为再生天数（每夜 -1 倒计时，减到 -1 可再收获）
            let regrow_days = regrow_days_for(&name);
            modified = modified.replace(
                &format!("<dayOfCurrentPhase>{}</dayOfCurrentPhase>", dop),
                &format!("<dayOfCurrentPhase>{}</dayOfCurrentPhase>", regrow_days),
            );
        } else if let Some((cs, ce)) = block.find("<crop>")
            .and_then(|s| block.find("</crop>").map(|e| (s, e + "</crop>".len())))
            .filter(|(s, e)| s < e)
        {
            modified = format!("{}{}", &block[..cs], &block[ce..]);
        }
        *seg = modified;
    }

    let mut details: Vec<String> = by_name.iter()
        .map(|(n, (c, p))| format!("{} ×{} ({}g/个, {}g)", n, c, p, c * (*p as u32)))
        .collect();
    details.sort();
    if details.is_empty() {
        details.push("没有可收获的成熟作物".to_string());
    } else {
        details.push(format!("总收益: {}g", total_income));
    }
    (segments.join(""), total_count, details, money + total_income)
}

fn clear_dead(xml: &str, money: i32) -> (String, u32, Vec<String>, i32) {
    let mut segments = split_blocks(xml);
    let mut count = 0u32;

    for seg in segments.iter_mut() {
        if !seg.starts_with(HOE_DIRT_OPEN) || !seg.contains("<dead>true</dead>") {
            continue;
        }
        if let Some((cs, ce)) = seg.find("<crop>")
            .and_then(|s| seg.find("</crop>").map(|e| (s, e + "</crop>".len())))
            .filter(|(s, e)| s < e)
        {
            *seg = format!("{}{}", &seg[..cs], &seg[ce..]);
            count += 1;
        }
    }
    (segments.join(""), count, vec![format!("已清理 {} 株枯死作物", count)], money)
}

fn extract_int(block: &str, tag: &str) -> Option<i32> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let p = block.find(&open)?;
    let after = &block[p + open.len()..];
    let e = after.find(&close)?;
    after[..e].trim().parse().ok()
}

/// 再生作物的再生天数（官方 Data/Crops 值）
fn regrow_days_for(name: &str) -> i32 {
    match name {
        "青豆" => 3,
        "草莓" => 4,
        "辣椒" => 3,
        "啤酒花" => 3,
        "番茄" => 4,
        "蓝莓" => 4,
        "西葫芦" => 3,
        "玉米" => 4,
        "茄子" => 5,
        "蔓越莓" => 5,
        "葡萄" => 3,
        "咖啡豆" => 2,
        "远古水果" => 7,
        "茶叶" => 5,
        _ => 4,
    }
}

