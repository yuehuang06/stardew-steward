// fetch_wiki 工具: 通过 MediaWiki API 查询星露谷 wiki
// LLM 发现本地知识库查不到的数据时主动调用，结果可缓存到本地

use serde::Serialize;

#[derive(Serialize)]
struct WikiResult {
    title: String,
    summary: String,
    source: String,
}

/// 通过 MediaWiki API 搜索星露谷 wiki 并提取摘要
pub async fn execute(keyword: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    // 1. 搜索页面标题
    let search_url = "https://stardewvalleywiki.com/mediawiki/api.php";
    let search_resp: serde_json::Value = client
        .get(search_url)
        .query(&[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", keyword),
            ("srlimit", "3"),
            ("format", "json"),
        ])
        .send()
        .await?
        .json()
        .await?;

    let search_results = &search_resp["query"]["search"];
    let results = search_results
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("wiki 搜索返回异常"))?;

    if results.is_empty() {
        return Ok(format!("wiki 上未找到与「{}」相关的内容。", keyword));
    }

    // 2. 取第一个匹配页面的 wikitext
    let title = results[0]["title"]
        .as_str()
        .unwrap_or(keyword)
        .to_string();

    let parse_resp: serde_json::Value = client
        .get(search_url)
        .query(&[
            ("action", "parse"),
            ("page", &title),
            ("format", "json"),
            ("prop", "wikitext"),
            ("section", "0"),
        ])
        .send()
        .await?
        .json()
        .await?;

    let wikitext = parse_resp["parse"]["wikitext"]["*"]
        .as_str()
        .unwrap_or("");

    // 3. 从 wikitext 提取关键信息
    let summary = extract_wiki_info(wikitext, &title);

    let result = WikiResult {
        title: title.clone(),
        summary,
        source: format!("stardewvalleywiki.com - {}", title),
    };

    Ok(serde_json::to_string_pretty(&result)?)
}

/// 从 wikitext 提取关键信息（Infobox 模板 + 首段文字）
fn extract_wiki_info(wikitext: &str, title: &str) -> String {
    let mut info = Vec::new();

    // 解析 Infobox 模板字段
    let infobox_start = wikitext.find("{{Infobox");
    if let Some(start) = infobox_start {
        // 找到匹配的 }}
        let info_section = &wikitext[start..];
        let end = info_section.find("}}").unwrap_or(info_section.len());
        let infobox = &info_section[..end];

        for line in infobox.lines() {
            let line = line.trim();
            if line.starts_with('|') {
                let line = &line[1..];
                if let Some(eq) = line.find('=') {
                    let key = line[..eq].trim();
                    let val = clean_wiki_markup(line[eq + 1..].trim());
                    if !val.is_empty() && is_useful_field(key) {
                        info.push(format!("{}: {}", key, val));
                    }
                }
            }
        }
    }

    // 提取首段文字（Infobox 之后的第一段非空文本）
    let after_infobox = wikitext.find("}}").map(|i| &wikitext[i + 2..]).unwrap_or(wikitext);
    for para in after_infobox.split("\n\n") {
        let cleaned = clean_wiki_markup(para.trim());
        if cleaned.len() > 20 && !cleaned.starts_with("{{") && !cleaned.starts_with("==") {
            info.push(format!("描述: {}", cleaned));
            break;
        }
    }

    if info.is_empty() {
        format!("无法从 wiki 页面「{}」提取结构化信息。", title)
    } else {
        info.join("\n")
    }
}

/// 清理 wiki 标记（{{...}}, [[...]], '''...''', <span> 等）
fn clean_wiki_markup(text: &str) -> String {
    let mut result = text.to_string();

    // 移除 [[...]] 的内部链接，保留显示文本
    while let (Some(start), Some(end)) = (result.find("[["), result.find("]]")) {
        if start < end {
            let link = &result[start + 2..end];
            // [[A|B]] -> B, [[A]] -> A
            let display = link.split('|').last().unwrap_or(link);
            result = format!("{}{}{}", &result[..start], display, &result[end + 2..]);
        } else {
            break;
        }
    }

    // 移除 {{...}} 模板，保留模板名后的文本
    while let Some(start) = result.find("{{") {
        if let Some(end) = result.find("}}") {
            if start < end {
                let template = &result[start + 2..end];
                // {{Name|Item}} -> Item
                let parts: Vec<&str> = template.split('|').collect();
                let display = if parts.len() >= 2 { parts[1] } else { "" };
                result = format!("{}{}{}", &result[..start], display, &result[end + 2..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // 移除 ''' 加粗标记
    result = result.replace("'''", "");
    // 移除 '' 斜体标记
    result = result.replace("''", "");
    // 移除 HTML 标签
    while let (Some(start), Some(end)) = (result.find('<'), result.find('>')) {
        if start < end {
            result = format!("{}{}", &result[..start], &result[end + 1..]);
        } else {
            break;
        }
    }

    result.trim().to_string()
}

/// 判断 Infobox 字段是否对玩家有用
fn is_useful_field(key: &str) -> bool {
    matches!(
        key.trim(),
        "eng" | "seed" | "growth" | "season" | "sellprice"
        | "edibility" | "color" | "xp" | "type" | "location"
        | "time" | "weather" | "difficulty" | "size" | "base"
    )
}
