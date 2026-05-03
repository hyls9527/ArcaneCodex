use crate::core::db::Database;
use crate::utils::error::AppResult;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: i64,
    pub pattern_name: String,
    pub pattern_description: Option<String>,
    pub occurrence_count: i64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordErrorPatternRequest {
    pub pattern_name: String,
    pub pattern_description: Option<String>,
}

#[tauri::command]
pub fn record_error_pattern(
    db: State<'_, Database>,
    request: RecordErrorPatternRequest,
) -> AppResult<i64> {
    let conn = db.open_connection()?;

    let row_id = conn.execute(
        "INSERT OR REPLACE INTO error_patterns (pattern_name, pattern_description, occurrence_count, last_seen) 
         VALUES (?1, ?2, COALESCE((SELECT occurrence_count FROM error_patterns WHERE pattern_name = ?1), 0) + 1, CURRENT_TIMESTAMP)",
        params![request.pattern_name, request.pattern_description],
    )?;

    Ok(row_id as i64)
}

#[tauri::command]
pub fn get_error_patterns(
    db: State<'_, Database>,
    limit: Option<usize>,
    min_occurrences: Option<i64>,
) -> AppResult<Vec<ErrorPattern>> {
    let conn = db.open_connection()?;
    let limit = limit.unwrap_or(50);
    let min_occ = min_occurrences.unwrap_or(1);

    let mut stmt = conn.prepare(
        "SELECT id, pattern_name, pattern_description, occurrence_count, first_seen, last_seen 
         FROM error_patterns 
         WHERE occurrence_count >= ?1 
         ORDER BY occurrence_count DESC 
         LIMIT ?2",
    )?;

    let patterns = stmt
        .query_map(params![min_occ, limit], |row| {
            Ok(ErrorPattern {
                id: row.get(0)?,
                pattern_name: row.get(1)?,
                pattern_description: row.get(2)?,
                occurrence_count: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(patterns)
}

#[tauri::command]
pub fn check_error_pattern_exists(
    db: State<'_, Database>,
    pattern_name: String,
) -> AppResult<Option<ErrorPattern>> {
    let conn = db.open_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, pattern_name, pattern_description, occurrence_count, first_seen, last_seen 
         FROM error_patterns 
         WHERE pattern_name = ?1",
    )?;

    let pattern = stmt
        .query_row(params![pattern_name], |row| {
            Ok(ErrorPattern {
                id: row.get(0)?,
                pattern_name: row.get(1)?,
                pattern_description: row.get(2)?,
                occurrence_count: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
            })
        })
        .ok();

    Ok(pattern)
}

#[tauri::command]
pub fn delete_error_pattern(db: State<'_, Database>, pattern_id: i64) -> AppResult<()> {
    let conn = db.open_connection()?;

    conn.execute(
        "DELETE FROM error_patterns WHERE id = ?1",
        params![pattern_id],
    )?;

    Ok(())
}

#[tauri::command]
pub fn get_high_frequency_error_patterns(
    db: State<'_, Database>,
    min_count: Option<i64>,
) -> AppResult<Vec<ErrorPattern>> {
    get_error_patterns(db, Some(20), min_count)
}

#[cfg(test)]
mod tests {
    use crate::core::db::Database;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_error_patterns.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
        (db, temp_dir)
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_record_error_pattern_success() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_record_error_pattern_increment_count() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_get_error_patterns_empty() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_check_error_pattern_exists() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_delete_error_pattern() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "需要 Tauri State 运行时环境"]
    fn test_get_high_frequency_error_patterns() {
        let (db, _temp) = setup_test_db();
        let _ = db;
    }

    #[test]
    #[ignore = "v1.1: 需要积累 50+ 错误模式记录后实现 CalibrationService 集成"]
    fn test_error_pattern_feedback_to_calibration() {
        // v1.1 路线图: 错误模式反馈到置信度校准
        //
        // 功能需求:
        // 1. 分析 error_patterns 表中的高频错误模式
        // 2. 将错误模式与 CalibrationService 关联
        // 3. 根据错误模式调整置信度校准曲线
        // 4. 例如: 如果 "动物误识别为物品" 是高频错误,
        //    则降低动物类别的原始置信度权重
        //
        // 前置条件:
        // - 需要积累至少 50 个错误模式记录
        // - 需要实现错误模式分类 (如: 误分类、漏检、重复等)
        // - 需要与 CalibrationService 集成
        //
        // 实现思路:
        // - 定期分析 error_patterns 表
        // - 根据错误类型调整对应类别的校准曲线
        // - 在 AI 推理时应用调整后的置信度
        // - 监控调整后准确率是否提升
    }
}
