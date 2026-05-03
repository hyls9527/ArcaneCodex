use crate::core::db::Database;
use crate::utils::error::{AppResult, AppError};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTagCorrectionRequest {
    pub image_id: i64,
    pub old_tags: Vec<String>,
    pub new_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCorrectionRecord {
    pub id: i64,
    pub image_id: i64,
    pub old_tags: Vec<String>,
    pub new_tags: Vec<String>,
    pub corrected_at: String,
}

#[tauri::command]
pub fn record_tag_correction(
    db: State<'_, Database>,
    request: RecordTagCorrectionRequest,
) -> AppResult<i64> {
    if request.old_tags.is_empty() && request.new_tags.is_empty() {
        return Err(AppError::validation("old_tags 和 new_tags 不能同时为空".to_string()));
    }

    let conn = db.open_connection()?;

    let old_tags_json = serde_json::to_string(&request.old_tags)
        .map_err(|e| AppError::validation(format!("old_tags 序列化失败: {}", e)))?;
    let new_tags_json = serde_json::to_string(&request.new_tags)
        .map_err(|e| AppError::validation(format!("new_tags 序列化失败: {}", e)))?;

    let row_id = conn.execute(
        "INSERT INTO tag_corrections (image_id, old_tags, new_tags) VALUES (?1, ?2, ?3)",
        params![request.image_id, old_tags_json, new_tags_json],
    )?;

    conn.execute(
        "UPDATE images SET ai_tags = ?1 WHERE id = ?2",
        params![new_tags_json, request.image_id],
    )?;

    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO error_patterns (pattern_name, pattern_description, occurrence_count, last_seen) 
         VALUES (?1, ?2, COALESCE((SELECT occurrence_count FROM error_patterns WHERE pattern_name = ?1), 0) + 1, CURRENT_TIMESTAMP)"
    )?;

    let pattern_name = format!("tag_correction_{}", request.image_id);
    let pattern_desc = format!("用户修正了 image_id={} 的标签，从 {:?} 改为 {:?}",
        request.image_id, request.old_tags, request.new_tags);
    
    let _ = stmt.execute(params![pattern_name, pattern_desc]);

    Ok(row_id as i64)
}

#[tauri::command]
pub fn get_tag_correction_history(
    db: State<'_, Database>,
    image_id: i64,
) -> AppResult<Vec<TagCorrectionRecord>> {
    let conn = db.open_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, image_id, old_tags, new_tags, corrected_at 
         FROM tag_corrections 
         WHERE image_id = ?1 
         ORDER BY corrected_at DESC"
    )?;

    let records = stmt
        .query_map(params![image_id], |row| {
            let old_tags_json: String = row.get(2)?;
            let new_tags_json: String = row.get(3)?;

            let old_tags: Vec<String> = serde_json::from_str(&old_tags_json)
                .unwrap_or_default();
            let new_tags: Vec<String> = serde_json::from_str(&new_tags_json)
                .unwrap_or_default();

            Ok(TagCorrectionRecord {
                id: row.get(0)?,
                image_id: row.get(1)?,
                old_tags,
                new_tags,
                corrected_at: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

#[tauri::command]
pub fn get_all_tag_corrections(
    db: State<'_, Database>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> AppResult<Vec<TagCorrectionRecord>> {
    let conn = db.open_connection()?;
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT id, image_id, old_tags, new_tags, corrected_at 
         FROM tag_corrections 
         ORDER BY corrected_at DESC 
         LIMIT ?1 OFFSET ?2"
    )?;

    let records = stmt
        .query_map(params![limit, offset], |row| {
            let old_tags_json: String = row.get(2)?;
            let new_tags_json: String = row.get(3)?;

            let old_tags: Vec<String> = serde_json::from_str(&old_tags_json)
                .unwrap_or_default();
            let new_tags: Vec<String> = serde_json::from_str(&new_tags_json)
                .unwrap_or_default();

            Ok(TagCorrectionRecord {
                id: row.get(0)?,
                image_id: row.get(1)?,
                old_tags,
                new_tags,
                corrected_at: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Database;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_tag_correction.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO images (id, file_path, file_name, file_size, ai_status) 
             VALUES (1, '/test/image.jpg', 'image.jpg', 12345, 'completed')",
            [],
        ).unwrap();

        (db, temp_dir)
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_record_tag_correction_success() {
        let (db, _temp) = setup_test_db();

        let request = RecordTagCorrectionRequest {
            image_id: 1,
            old_tags: vec!["狗".to_string(), "宠物".to_string()],
            new_tags: vec!["猫".to_string(), "宠物".to_string()],
        };

        // 注意：tauri::State 不能直接从 Database 创建，需要 Tauri 运行时
        // 此测试需要集成测试环境
        let _ = db; // 使用变量避免警告
        let _ = request;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_record_tag_correction_empty_tags() {
        let (db, _temp) = setup_test_db();

        let request = RecordTagCorrectionRequest {
            image_id: 1,
            old_tags: vec![],
            new_tags: vec![],
        };

        let _ = db;
        let _ = request;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_get_tag_correction_history() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_get_all_tag_corrections() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "v1.1: 需要积累 100+ 修正记录后实现 prompt 微调"]
    fn test_prompt_fine_tuning_with_corrections() {
        // v1.1 路线图: 定期用修正样本微调 prompt
        // 
        // 功能需求:
        // 1. 定期分析 tag_corrections 表中的修正记录
        // 2. 识别常见的标签修正模式
        // 3. 根据修正模式调整 AI prompt
        // 4. 例如: 如果用户经常把 "狗" 修正为 "猫"，
        //    则在 prompt 中强调 "仔细区分狗和猫"
        //
        // 前置条件:
        // - 需要积累至少 100 条修正记录
        // - 需要实现 prompt 版本管理
        // - 需要 A/B 测试验证新 prompt 效果
        //
        // 实现思路:
        // - 使用统计分析找出高频修正模式
        // - 使用 LLM 生成针对性的 prompt 调整建议
        // - 人工审核后应用到生产环境
    }
}
