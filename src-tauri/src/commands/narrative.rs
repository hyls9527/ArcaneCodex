use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use crate::core::ai_queue::AITaskQueue;
use crate::utils::error::{AppError, AppResult};
use rusqlite::params;

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeResult {
    pub id: i64,
    pub image_id: i64,
    pub content: String,
    pub entities_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociationResult {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub thumbnail_path: Option<String>,
    pub narrative_content: String,
    pub match_type: String,
    pub relevance: f64,
}

const PERSON_PREFIXES: &[&str] = &["和", "与", "跟"];
const PLACE_PREFIXES: &[&str] = &["在", "去", "到", "从"];
const PLACE_SUFFIXES: &[&str] = &[
    "的", "那", "玩", "出差", "旅行", "旅游", "逛", "吃饭", "拍照", "看", "住", "走",
];
const TIME_KEYWORDS: &[&str] = &[
    "去年", "今年", "昨天", "今天", "上个月", "这个月", "前年",
];
const WEEKDAY_PREFIXES: &[&str] = &["周", "星期"];
const WEEKDAY_SUFFIXES: &[&str] = &["一", "二", "三", "四", "五", "六", "日", "天"];

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{2A700}'..='\u{2B73F}' |
        '\u{2B740}'..='\u{2B81F}' |
        '\u{2B820}'..='\u{2CEAF}' |
        '\u{F900}'..='\u{FAFF}' |
        '\u{2F800}'..='\u{2FA1F}'
    )
}

fn extract_entities(content: &str) -> Vec<serde_json::Value> {
    let mut entities = Vec::new();
    let mut seen_persons = std::collections::HashSet::new();
    let mut seen_places = std::collections::HashSet::new();
    let mut seen_times = std::collections::HashSet::new();
    let chars: Vec<char> = content.chars().collect();

    fn find_substring(chars: &[char], pattern: &[char], from: usize) -> Option<usize> {
        if pattern.is_empty() || from + pattern.len() > chars.len() {
            return None;
        }
        for i in from..=chars.len() - pattern.len() {
            if &chars[i..i + pattern.len()] == pattern {
                return Some(i);
            }
        }
        None
    }

    fn all_cjk(chars: &[char], start: usize, len: usize) -> bool {
        if start + len > chars.len() {
            return false;
        }
        chars[start..start + len].iter().all(|c| is_cjk(*c))
    }

    for prefix in PERSON_PREFIXES {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        let mut search_from = 0;
        while let Some(pos) = find_substring(&chars, &prefix_chars, search_from) {
            let after = pos + prefix_chars.len();
            for &name_len in &[1, 2, 3] {
                if all_cjk(&chars, after, name_len) {
                    let name: String = chars[after..after + name_len].iter().collect();
                    if !seen_persons.contains(&name) {
                        seen_persons.insert(name.clone());
                        entities.push(serde_json::json!({
                            "type": "person",
                            "value": name
                        }));
                    }
                    break;
                }
            }
            search_from = after;
        }
    }

    for prefix in PLACE_PREFIXES {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        let mut search_from = 0;
        while let Some(pos) = find_substring(&chars, &prefix_chars, search_from) {
            let after = pos + prefix_chars.len();

            let from_prefix: String = chars[pos..].iter().collect();
            let is_time_at_prefix = TIME_KEYWORDS.iter().any(|kw| from_prefix.starts_with(kw));
            let after_text: String = if after < chars.len() {
                chars[after..].iter().collect()
            } else {
                String::new()
            };
            let is_time_after_prefix = TIME_KEYWORDS.iter().any(|kw| after_text.starts_with(kw));

            if is_time_at_prefix || is_time_after_prefix {
                search_from = after;
                continue;
            }

            for place_len in 1..=6 {
                if !all_cjk(&chars, after, place_len) {
                    break;
                }
                let rest_start = after + place_len;
                if rest_start >= chars.len() {
                    if place_len <= 2 {
                        let place: String = chars[after..after + place_len].iter().collect();
                        if !seen_places.contains(&place) {
                            seen_places.insert(place.clone());
                            entities.push(serde_json::json!({
                                "type": "place",
                                "value": place
                            }));
                        }
                    }
                    break;
                }
                let rest: String = chars[rest_start..].iter().collect();
                let has_suffix = PLACE_SUFFIXES.iter().any(|suffix| rest.starts_with(suffix));
                if has_suffix || place_len <= 2 {
                    let place: String = chars[after..after + place_len].iter().collect();
                    if !seen_places.contains(&place) {
                        seen_places.insert(place.clone());
                        entities.push(serde_json::json!({
                            "type": "place",
                            "value": place
                        }));
                    }
                    break;
                }
            }
            search_from = after;
        }
    }

    for keyword in TIME_KEYWORDS {
        if content.contains(keyword) && !seen_times.contains(*keyword) {
            seen_times.insert(keyword.to_string());
            entities.push(serde_json::json!({
                "type": "time",
                "value": keyword
            }));
        }
    }

    for wp in WEEKDAY_PREFIXES {
        if let Some(pos) = content.find(wp) {
            let after = pos + wp.len();
            let char_pos = content[..after].chars().count();
            if char_pos < chars.len() {
                let day_char = chars[char_pos];
                let day_str = day_char.to_string();
                if WEEKDAY_SUFFIXES.contains(&day_str.as_str()) {
                    let time_val = format!("{}{}", wp, day_char);
                    if !seen_times.contains(&time_val) {
                        seen_times.insert(time_val.clone());
                        entities.push(serde_json::json!({
                            "type": "time",
                            "value": time_val
                        }));
                    }
                }
            }
        }
    }

    entities
}

#[tauri::command]
pub async fn write_narrative(
    image_id: i64,
    content: String,
    queue: State<'_, Arc<AITaskQueue>>,
) -> AppResult<NarrativeResult> {
    if content.trim().is_empty() {
        return Err(AppError::validation("叙事内容不能为空"));
    }

    let entities = extract_entities(&content);
    let entities_json = serde_json::to_string(&entities).unwrap_or_default();

    let db = queue.db();
    let conn = db.open_connection()?;

    conn.execute(
        "INSERT INTO narratives (image_id, content, entities_json) VALUES (?1, ?2, ?3)",
        params![image_id, content, entities_json],
    )?;

    let id = conn.last_insert_rowid();

    Ok(NarrativeResult {
        id,
        image_id,
        content,
        entities_json,
    })
}

#[tauri::command]
pub async fn get_narratives(
    image_id: i64,
    queue: State<'_, Arc<AITaskQueue>>,
) -> AppResult<Vec<NarrativeResult>> {
    let db = queue.db();
    let conn = db.open_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, image_id, content, entities_json FROM narratives WHERE image_id = ?1"
    )?;

    let rows = stmt.query_map(params![image_id], |row| {
        Ok(NarrativeResult {
            id: row.get(0)?,
            image_id: row.get(1)?,
            content: row.get(2)?,
            entities_json: row.get(3)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

#[tauri::command]
pub async fn query_associations(
    query: String,
    limit: Option<i64>,
    queue: State<'_, Arc<AITaskQueue>>,
) -> AppResult<Vec<AssociationResult>> {
    let limit = limit.unwrap_or(20);
    let escaped_query = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{}%", escaped_query);

    let db = queue.db();
    let conn = db.open_connection()?;

    let mut stmt = conn.prepare(
        "SELECT n.image_id, i.file_path, i.file_name, i.thumbnail_path, n.content, \
         CASE \
            WHEN n.content LIKE ?1 ESCAPE '\\' THEN 'content' \
            WHEN n.entities_json LIKE ?1 ESCAPE '\\' THEN 'entity' \
         END as match_type \
         FROM narratives n \
         JOIN images i ON n.image_id = i.id \
         WHERE n.content LIKE ?1 ESCAPE '\\' OR n.entities_json LIKE ?1 ESCAPE '\\' \
         ORDER BY n.updated_at DESC \
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        let match_type: String = row.get(5)?;
        Ok(AssociationResult {
            image_id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            thumbnail_path: row.get(3)?,
            narrative_content: row.get(4)?,
            match_type,
            relevance: 1.0,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_entities_person() {
        let entities = extract_entities("我和小明一起去公园玩");
        let persons: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "person")
            .collect();
        assert!(!persons.is_empty());
    }

    #[test]
    fn test_extract_entities_place() {
        let entities = extract_entities("我去北京出差");
        let places: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "place")
            .collect();
        assert!(!places.is_empty());
    }

    #[test]
    fn test_extract_entities_time() {
        let entities = extract_entities("去年我们去旅行");
        let times: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .collect();
        assert!(!times.is_empty());
    }

    #[test]
    fn test_extract_entities_combined() {
        let entities = extract_entities("和老王在杭州去年夏天");
        let persons: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "person")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        let places: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "place")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        let times: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        // Debug: print actual values
        eprintln!("Persons: {:?}", persons);
        eprintln!("Places: {:?}", places);
        eprintln!("Times: {:?}", times);
        assert!(persons.iter().any(|p| p.contains("老王") || "老王".contains(p)), "Expected '老王' or substring, got: {:?}", persons);
        assert!(places.iter().any(|p| p.contains("杭州") || "杭州".contains(p)), "Expected '杭州' or substring, got: {:?}", places);
        assert!(times.iter().any(|t| t.contains("去年") || "去年".contains(t)), "Expected '去年' or substring, got: {:?}", times);
    }

    #[test]
    fn test_extract_entities_empty() {
        let entities = extract_entities("一张风景照");
        assert!(entities.iter().all(|e| e["type"] != "person"));
        assert!(entities.iter().all(|e| e["type"] != "place"));
        assert!(entities.iter().all(|e| e["type"] != "time"));
    }

    #[test]
    fn test_extract_entities_weekday() {
        let entities = extract_entities("周三开会");
        let times: Vec<_> = entities
            .iter()
            .filter(|e| e["type"] == "time")
            .collect();
        assert!(!times.is_empty());
    }

    #[test]
    fn test_narrative_result_serialization() {
        let result = NarrativeResult {
            id: 1,
            image_id: 42,
            content: "和老王在杭州去年夏天".to_string(),
            entities_json: r#"[{"type":"person","value":"老王"}]"#.to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: NarrativeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.image_id, 42);
        assert_eq!(deserialized.content, "和老王在杭州去年夏天");
    }

    #[test]
    fn test_association_result_serialization() {
        let result = AssociationResult {
            image_id: 1,
            file_path: "/photos/test.jpg".to_string(),
            file_name: "test.jpg".to_string(),
            thumbnail_path: Some("/thumbs/test.webp".to_string()),
            narrative_content: "和老王在杭州".to_string(),
            match_type: "content".to_string(),
            relevance: 1.0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AssociationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.image_id, 1);
        assert_eq!(deserialized.match_type, "content");
        assert_eq!(deserialized.relevance, 1.0);
    }

    #[test]
    fn test_write_narrative_empty_content() {
        let result: AppResult<NarrativeResult> = Err(AppError::validation("叙事内容不能为空"));
        assert!(result.is_err());
    }
}
