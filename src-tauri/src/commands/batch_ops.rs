#![allow(missing_docs)]
use crate::commands::export::escape_csv_field;
use crate::commands::images::sanitize_path;
use crate::core::ai_queue::AITaskQueue;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tauri::State;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAITagRequest {
    pub image_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskStatus {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub in_progress: usize,
}

#[tauri::command]
pub fn start_batch_ai_tag(
    db: State<'_, Database>,
    ai_queue: State<'_, AITaskQueue>,
    request: BatchAITagRequest,
) -> AppResult<usize> {
    if request.image_ids.is_empty() {
        return Err(AppError::validation("image_ids 不能为空".to_string()));
    }

    let conn = db.open_connection()?;

    let mut enqueued_count = 0;
    for chunk in request.image_ids.chunks(500) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, file_path FROM images WHERE id IN ({}) AND ai_status IN ('pending', 'failed')",
            placeholders
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok());

        for (id, path) in rows {
            ai_queue
                .add_task(id, &path)
                .map_err(|e| AppError::ai(e.to_string()))?;
            enqueued_count += 1;
        }
    }

    Ok(enqueued_count)
}

#[tauri::command]
pub fn get_batch_ai_status(ai_queue: State<'_, AITaskQueue>) -> AppResult<BatchTaskStatus> {
    let stats = ai_queue.get_stats();

    Ok(BatchTaskStatus {
        total: stats.get("total").copied().unwrap_or(0),
        completed: stats.get("completed").copied().unwrap_or(0),
        failed: stats.get("failed").copied().unwrap_or(0),
        in_progress: stats.get("in_progress").copied().unwrap_or(0),
    })
}

#[tauri::command]
pub fn pause_batch_ai_task(ai_queue: State<'_, AITaskQueue>) -> AppResult<()> {
    ai_queue.pause();
    Ok(())
}

#[tauri::command]
pub fn resume_batch_ai_task(ai_queue: State<'_, AITaskQueue>) -> AppResult<()> {
    ai_queue.resume();
    Ok(())
}

#[tauri::command]
pub fn cancel_batch_ai_task(ai_queue: State<'_, AITaskQueue>) -> AppResult<usize> {
    let cancelled = ai_queue.clear_pending();
    Ok(cancelled)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TagOperation {
    Add,
    Remove,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTagCorrectionRequest {
    pub image_ids: Vec<i64>,
    pub tags: Vec<String>,
    pub operation: TagOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTagCorrectionResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failed_ids: Vec<i64>,
}

#[tauri::command]
pub fn batch_tag_correction(
    db: State<'_, Database>,
    request: BatchTagCorrectionRequest,
) -> AppResult<BatchTagCorrectionResult> {
    if request.image_ids.is_empty() {
        return Err(AppError::validation("image_ids 不能为空".to_string()));
    }

    if request.tags.is_empty() && request.operation != TagOperation::Remove {
        return Err(AppError::validation("tags 不能为空".to_string()));
    }

    let conn = db.open_connection()?;
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut failed_ids = Vec::new();

    for &image_id in &request.image_ids {
        match apply_tag_operation(&conn, image_id, &request.tags, &request.operation) {
            Ok(_) => success_count += 1,
            Err(e) => {
                warn!("标签操作失败 (image_id={}): {}", image_id, e);
                failed_count += 1;
                failed_ids.push(image_id);
            }
        }
    }

    Ok(BatchTagCorrectionResult {
        success_count,
        failed_count,
        failed_ids,
    })
}

fn apply_tag_operation(
    conn: &rusqlite::Connection,
    image_id: i64,
    new_tags: &[String],
    operation: &TagOperation,
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;

    let current_tags_json: String = tx
        .query_row(
            "SELECT ai_tags FROM images WHERE id = ?1",
            rusqlite::params![image_id],
            |row| row.get(0),
        )
        .unwrap_or("[]".to_string());

    let mut current_tags: Vec<String> =
        serde_json::from_str(&current_tags_json).unwrap_or_default();

    let old_tags = current_tags.clone();

    let updated_tags = match operation {
        TagOperation::Add => {
            let mut merged = current_tags;
            for tag in new_tags {
                if !merged.contains(tag) {
                    merged.push(tag.clone());
                }
            }
            merged
        }
        TagOperation::Remove => {
            current_tags.retain(|tag| !new_tags.contains(tag));
            current_tags
        }
        TagOperation::Replace => new_tags.to_vec(),
    };

    let updated_tags_json = serde_json::to_string(&updated_tags)
        .map_err(|e| AppError::validation(format!("标签序列化失败: {}", e)))?;

    tx.execute(
        "UPDATE images SET ai_tags = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![updated_tags_json, image_id],
    )?;

    let old_tags_json = serde_json::to_string(&old_tags)
        .map_err(|e| AppError::validation(format!("旧标签序列化失败: {}", e)))?;

    tx.execute(
        "INSERT INTO tag_corrections (image_id, old_tags, new_tags) VALUES (?1, ?2, ?3)",
        rusqlite::params![image_id, old_tags_json, updated_tags_json],
    )?;

    tx.commit()?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExportRequest {
    pub image_ids: Vec<i64>,
    pub export_path: String,
    pub format: ExportFormat,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExportResult {
    pub exported_count: usize,
    pub export_path: String,
    pub file_size: u64,
}

#[tauri::command]
pub fn batch_export(
    db: State<'_, Database>,
    request: BatchExportRequest,
) -> AppResult<BatchExportResult> {
    if request.image_ids.is_empty() {
        return Err(AppError::validation("image_ids 不能为空".to_string()));
    }

    // 安全验证：防止路径遍历攻击
    let base_dir = get_user_writable_base_dir();
    let validated_path = sanitize_path(&base_dir, &request.export_path, false)
        .map_err(|e| AppError::validation(format!("导出路径无效: {}", e)))?;

    let conn = db.open_connection()?;

    let mut export_data = Vec::new();
    for chunk in request.image_ids.chunks(500) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, file_path, file_name, file_size, ai_tags, ai_description, ai_category, ai_confidence
             FROM images WHERE id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql)?;

        #[allow(clippy::type_complexity)]
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, f64>(7)?,
                ))
            })?
            .filter_map(|r| r.ok());

        for row in rows {
            let tags: Vec<String> = serde_json::from_str(&row.4).unwrap_or_default();
            export_data.push(BatchExportRecord {
                image_id: row.0,
                file_path: row.1,
                file_name: row.2,
                file_size: row.3,
                ai_tags: if request.include_metadata {
                    Some(tags)
                } else {
                    None
                },
                ai_description: if request.include_metadata {
                    Some(row.5)
                } else {
                    None
                },
                ai_category: if request.include_metadata {
                    Some(row.6)
                } else {
                    None
                },
                ai_confidence: if request.include_metadata {
                    Some(row.7)
                } else {
                    None
                },
            });
        }
    }

    let exported_count = export_data.len();

    match request.format {
        ExportFormat::Json => {
            let json_content = serde_json::to_string_pretty(&export_data)
                .map_err(|e| AppError::validation(format!("JSON 序列化失败: {}", e)))?;
            std::fs::write(&validated_path, json_content)
                .map_err(|e| AppError::validation(format!("文件写入失败: {}", e)))?;
        }
        ExportFormat::Csv => {
            let mut csv_content = String::from("id,file_path,file_name,file_size\n");
            for record in &export_data {
                csv_content.push_str(&format!(
                    "{},{},{},{}\n",
                    record.image_id,
                    escape_csv_field(&record.file_path),
                    escape_csv_field(&record.file_name),
                    record.file_size
                ));
            }
            std::fs::write(&validated_path, csv_content)
                .map_err(|e| AppError::validation(format!("文件写入失败: {}", e)))?;
        }
        ExportFormat::Xml => {
            let mut xml_content =
                String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<records>\n");
            for record in &export_data {
                xml_content.push_str(&format!(
                    "  <record>\n    <id>{}</id>\n    <file_path>{}</file_path>\n    <file_name>{}</file_name>\n    <file_size>{}</file_size>\n  </record>\n",
                    record.image_id, record.file_path, record.file_name, record.file_size
                ));
            }
            xml_content.push_str("</records>\n");
            std::fs::write(&validated_path, xml_content)
                .map_err(|e| AppError::validation(format!("文件写入失败: {}", e)))?;
        }
    }

    let file_size = std::fs::metadata(&validated_path)
        .map_err(|e| AppError::validation(format!("获取文件大小失败: {}", e)))?
        .len();

    Ok(BatchExportResult {
        exported_count,
        export_path: validated_path.to_string_lossy().to_string(),
        file_size,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchExportRecord {
    image_id: i64,
    file_path: String,
    file_name: String,
    file_size: i64,
    ai_tags: Option<Vec<String>>,
    ai_description: Option<String>,
    ai_category: Option<String>,
    ai_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_images: i64,
    pub category_distribution: Vec<(String, i64)>,
    pub ai_progress: AIProgressStats,
    pub storage_usage: StorageStats,
    pub tag_cloud: Vec<(String, i64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProgressStats {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
    pub verified: i64,
    pub provisional: i64,
    pub rejected: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_size_bytes: i64,
    pub average_image_size: i64,
    pub largest_image_size: i64,
}

#[tauri::command]
pub fn get_library_stats(db: State<'_, Database>) -> AppResult<LibraryStats> {
    let conn = db.open_connection()?;

    let total_images: i64 = conn
        .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
        .unwrap_or(0);

    let category_distribution: Vec<(String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(ai_category, 'uncategorized'), COUNT(*) 
             FROM images GROUP BY ai_category ORDER BY COUNT(*) DESC",
        )?;
        let results: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        results
    };

    let ai_progress: AIProgressStats = {
        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_status = 'pending'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let processing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_status = 'processing'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let completed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_status = 'completed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let failed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_status = 'failed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let verified: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_tag_status = 'verified'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let provisional: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_tag_status = 'provisional'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let rejected: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE ai_tag_status = 'rejected'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        AIProgressStats {
            pending,
            processing,
            completed,
            failed,
            verified,
            provisional,
            rejected,
        }
    };

    let storage_usage: StorageStats = {
        let total_size_bytes: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(file_size), 0) FROM images",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let average_image_size = if total_images > 0 {
            total_size_bytes / total_images
        } else {
            0
        };
        let largest_image_size: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(file_size), 0) FROM images",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        StorageStats {
            total_size_bytes,
            average_image_size,
            largest_image_size,
        }
    };

    let tag_cloud: Vec<(String, i64)> = {
        let mut stmt =
            conn.prepare("SELECT ai_tags FROM images WHERE ai_tags IS NOT NULL AND ai_tags != ''")?;
        let tag_rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut tag_counts = std::collections::HashMap::new();
        for tags_json in tag_rows {
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            for tag in tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }

        let mut tag_cloud: Vec<(String, i64)> = tag_counts.into_iter().collect();
        tag_cloud.sort_by_key(|x| std::cmp::Reverse(x.1));
        tag_cloud.truncate(50);
        tag_cloud
    };

    Ok(LibraryStats {
        total_images,
        category_distribution,
        ai_progress,
        storage_usage,
        tag_cloud,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyDataPoint {
    pub date: String,
    pub total: i64,
    pub correct: i64,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAccuracy {
    pub category: String,
    pub total: i64,
    pub verified: i64,
    pub provisional: i64,
    pub rejected: i64,
    pub average_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationComparison {
    pub before_ece: f64,
    pub after_ece: f64,
    pub improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyTrend {
    pub daily_data: Vec<AccuracyDataPoint>,
    pub category_accuracy: Vec<CategoryAccuracy>,
    pub calibration_comparison: Option<CalibrationComparison>,
}

#[tauri::command]
pub fn get_accuracy_trend(db: State<'_, Database>, days: Option<i64>) -> AppResult<AccuracyTrend> {
    let conn = db.open_connection()?;
    let days = days.unwrap_or(30);

    let daily_data: Vec<AccuracyDataPoint> = {
        let mut stmt = conn.prepare(
            "SELECT 
                DATE(ai_processed_at) as date,
                COUNT(*) as total,
                SUM(CASE WHEN ai_tag_status = 'verified' THEN 1 ELSE 0 END) as correct
             FROM images 
             WHERE ai_status = 'completed' 
               AND ai_processed_at IS NOT NULL
               AND ai_processed_at >= DATE('now', ?1)
             GROUP BY DATE(ai_processed_at)
             ORDER BY date ASC",
        )?;

        // 安全说明：days 是 i64 类型，format! 仅用于构建 SQLite DATE 函数参数
        // 最终通过参数化查询传递，不存在 SQL 注入风险
        let days_param = format!("-{} days", days);

        let results: Vec<AccuracyDataPoint> = stmt
            .query_map(rusqlite::params![days_param], |row| {
                let total: i64 = row.get(1)?;
                let correct: i64 = row.get(2)?;
                let accuracy = if total > 0 {
                    correct as f64 / total as f64
                } else {
                    0.0
                };

                Ok(AccuracyDataPoint {
                    date: row.get(0)?,
                    total,
                    correct,
                    accuracy,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        results
    };

    let category_accuracy: Vec<CategoryAccuracy> = {
        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(ai_category, 'uncategorized') as category,
                COUNT(*) as total,
                SUM(CASE WHEN ai_tag_status = 'verified' THEN 1 ELSE 0 END) as verified,
                SUM(CASE WHEN ai_tag_status = 'provisional' THEN 1 ELSE 0 END) as provisional,
                SUM(CASE WHEN ai_tag_status = 'rejected' THEN 1 ELSE 0 END) as rejected,
                COALESCE(AVG(ai_confidence), 0.0) as average_confidence
             FROM images 
             WHERE ai_status = 'completed'
             GROUP BY ai_category
             ORDER BY total DESC",
        )?;

        let results: Vec<CategoryAccuracy> = stmt
            .query_map([], |row| {
                Ok(CategoryAccuracy {
                    category: row.get(0)?,
                    total: row.get(1)?,
                    verified: row.get(2)?,
                    provisional: row.get(3)?,
                    rejected: row.get(4)?,
                    average_confidence: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        results
    };

    let calibration_comparison = {
        let latest_report: Option<(f64, String)> = conn
            .query_row(
                "SELECT overall_ece, computed_at FROM calibration_reports 
             ORDER BY created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        let previous_report: Option<(f64, String)> = conn
            .query_row(
                "SELECT overall_ece, computed_at FROM calibration_reports 
             ORDER BY created_at DESC LIMIT 1 OFFSET 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        match (latest_report, previous_report) {
            (Some((after_ece, _)), Some((before_ece, _))) => {
                let improvement = if before_ece > 0.0 {
                    ((before_ece - after_ece) / before_ece) * 100.0
                } else {
                    0.0
                };
                Some(CalibrationComparison {
                    before_ece,
                    after_ece,
                    improvement_percent: improvement.max(0.0),
                })
            }
            _ => None,
        }
    };

    Ok(AccuracyTrend {
        daily_data,
        category_accuracy,
        calibration_comparison,
    })
}

fn get_log_path() -> String {
    std::env::var("APPDATA")
        .map(|appdata| format!("{}\\ArcaneCodex\\logs\\app.log", appdata))
        .unwrap_or_else(|_| "./logs/app.log".to_string())
}

/// 获取用户可写的基础目录，用于路径遍历防护
/// 优先级：USERPROFILE > HOME > 当前目录
fn get_user_writable_base_dir() -> PathBuf {
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileStats {
    pub path: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResponse {
    pub entries: Vec<LogEntry>,
    pub total_lines: usize,
    pub has_more: bool,
}

#[tauri::command]
pub fn get_log_entries(
    max_lines: Option<usize>,
    offset: Option<usize>,
    level_filter: Option<String>,
) -> AppResult<LogResponse> {
    let log_path = get_log_path();
    let max_lines = max_lines.unwrap_or(200);
    let offset = offset.unwrap_or(0);

    if !std::path::Path::new(&log_path).exists() {
        return Ok(LogResponse {
            entries: vec![],
            total_lines: 0,
            has_more: false,
        });
    }

    let file = std::fs::File::open(&log_path)
        .map_err(|e| AppError::internal(format!("无法打开日志文件: {}", e)))?;
    let reader = BufReader::new(file);

    let mut entries: Vec<LogEntry> = Vec::new();
    let mut line_count = 0;

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        line_count += 1;

        if line_count <= offset {
            continue;
        }

        if entries.len() >= max_lines {
            break;
        }

        if let Some(ref filter) = level_filter {
            let upper_filter = filter.to_uppercase();
            if !line.contains(&upper_filter) {
                continue;
            }
        }

        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        let entry = if parts.len() >= 4 {
            LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].trim_matches(|c| c == '{' || c == '}').to_string(),
                target: parts[2].to_string(),
                message: parts[3..].join(" ").to_string(),
            }
        } else {
            LogEntry {
                timestamp: String::new(),
                level: "UNKNOWN".to_string(),
                target: String::new(),
                message: line,
            }
        };

        entries.push(entry);
    }

    let has_more = line_count > (offset + entries.len());

    Ok(LogResponse {
        entries,
        total_lines: line_count,
        has_more,
    })
}

#[tauri::command]
pub fn get_log_stats() -> AppResult<LogFileStats> {
    let log_path = get_log_path();
    let path = std::path::Path::new(&log_path);

    if !path.exists() {
        return Ok(LogFileStats {
            path: log_path,
            size_bytes: 0,
            line_count: 0,
            exists: false,
        });
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| AppError::internal(format!("无法读取日志文件元数据: {}", e)))?;

    let file = std::fs::File::open(path)
        .map_err(|e| AppError::internal(format!("无法打开日志文件: {}", e)))?;
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();

    Ok(LogFileStats {
        path: log_path,
        size_bytes: metadata.len(),
        line_count,
        exists: true,
    })
}

#[tauri::command]
pub fn export_logs(export_path: String, level_filter: Option<String>) -> AppResult<usize> {
    let log_path = get_log_path();

    if !std::path::Path::new(&log_path).exists() {
        return Err(AppError::internal("日志文件不存在".to_string()));
    }

    // 安全验证：防止路径遍历攻击
    let base_dir = get_user_writable_base_dir();
    let validated_path = sanitize_path(&base_dir, &export_path, false)
        .map_err(|e| AppError::validation(format!("日志导出路径无效: {}", e)))?;

    let source = std::fs::File::open(&log_path)
        .map_err(|e| AppError::internal(format!("无法打开日志文件: {}", e)))?;
    let reader = BufReader::new(source);

    let target = std::fs::File::create(&validated_path)
        .map_err(|e| AppError::internal(format!("无法创建导出文件: {}", e)))?;
    let mut writer = std::io::BufWriter::new(target);

    let mut exported_count = 0;

    for line in reader.lines() {
        let line = line.unwrap_or_default();

        if let Some(ref filter) = level_filter {
            let upper_filter = filter.to_uppercase();
            if !line.contains(&upper_filter) {
                continue;
            }
        }

        writeln!(writer, "{}", line).ok();
        exported_count += 1;
    }

    writer.flush().ok();

    Ok(exported_count)
}

#[tauri::command]
pub fn clear_logs() -> AppResult<usize> {
    let log_path = get_log_path();
    let path = std::path::Path::new(&log_path);

    if !path.exists() {
        return Ok(0);
    }

    std::fs::write(path, "").map_err(|e| AppError::internal(format!("无法清空日志: {}", e)))?;

    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Database;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_batch_ops.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_batch_ai_tag_request_serialization() {
        let request = BatchAITagRequest {
            image_ids: vec![1, 2, 3],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("image_ids"));

        let deserialized: BatchAITagRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.image_ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_batch_task_status_serialization() {
        let status = BatchTaskStatus {
            total: 100,
            completed: 50,
            failed: 5,
            in_progress: 45,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: BatchTaskStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total, 100);
        assert_eq!(deserialized.completed, 50);
        assert_eq!(deserialized.failed, 5);
        assert_eq!(deserialized.in_progress, 45);
    }

    #[test]
    fn test_tag_operation_serialization() {
        let add_json = serde_json::to_string(&TagOperation::Add).unwrap();
        assert!(add_json.contains("Add"));

        let remove: TagOperation = serde_json::from_str("\"Remove\"").unwrap();
        assert!(matches!(remove, TagOperation::Remove));

        let replace: TagOperation = serde_json::from_str("\"Replace\"").unwrap();
        assert!(matches!(replace, TagOperation::Replace));
    }

    #[test]
    fn test_batch_tag_correction_request_validation() {
        let valid_request = BatchTagCorrectionRequest {
            image_ids: vec![1, 2],
            tags: vec!["风景".to_string(), "自然".to_string()],
            operation: TagOperation::Add,
        };
        assert!(!valid_request.image_ids.is_empty());
        assert!(!valid_request.tags.is_empty());

        let empty_tags_request = BatchTagCorrectionRequest {
            image_ids: vec![1],
            tags: vec![],
            operation: TagOperation::Remove,
        };
        assert!(empty_tags_request.tags.is_empty());
        assert!(matches!(empty_tags_request.operation, TagOperation::Remove));
    }

    #[test]
    fn test_batch_tag_correction_result_serialization() {
        let result = BatchTagCorrectionResult {
            success_count: 8,
            failed_count: 2,
            failed_ids: vec![3, 7],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BatchTagCorrectionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.success_count, 8);
        assert_eq!(deserialized.failed_count, 2);
        assert_eq!(deserialized.failed_ids, vec![3, 7]);
    }

    #[test]
    fn test_export_format_serialization() {
        let json_json = serde_json::to_string(&ExportFormat::Json).unwrap();
        assert!(json_json.contains("Json"));

        let csv: ExportFormat = serde_json::from_str("\"Csv\"").unwrap();
        assert!(matches!(csv, ExportFormat::Csv));

        let xml: ExportFormat = serde_json::from_str("\"Xml\"").unwrap();
        assert!(matches!(xml, ExportFormat::Xml));
    }

    #[test]
    fn test_batch_export_request_serialization() {
        let request = BatchExportRequest {
            image_ids: vec![1, 2, 3],
            export_path: "/tmp/export.json".to_string(),
            format: ExportFormat::Json,
            include_metadata: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: BatchExportRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.image_ids, vec![1, 2, 3]);
        assert_eq!(deserialized.export_path, "/tmp/export.json");
        assert!(matches!(deserialized.format, ExportFormat::Json));
        assert!(deserialized.include_metadata);
    }

    #[test]
    fn test_library_stats_structs() {
        let ai_progress = AIProgressStats {
            pending: 10,
            processing: 5,
            completed: 100,
            failed: 3,
            verified: 80,
            provisional: 15,
            rejected: 5,
        };

        let storage = StorageStats {
            total_size_bytes: 1073741824,
            average_image_size: 5242880,
            largest_image_size: 52428800,
        };

        let stats = LibraryStats {
            total_images: 118,
            category_distribution: vec![("风景".to_string(), 50), ("人物".to_string(), 40)],
            ai_progress,
            storage_usage: storage,
            tag_cloud: vec![("自然".to_string(), 30), ("户外".to_string(), 25)],
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: LibraryStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_images, 118);
        assert_eq!(deserialized.ai_progress.completed, 100);
        assert_eq!(deserialized.storage_usage.total_size_bytes, 1073741824);
        assert_eq!(deserialized.tag_cloud.len(), 2);
    }

    #[test]
    fn test_tag_operation_add_logic() {
        let current = vec!["标签1".to_string(), "标签2".to_string()];
        let new_tags = vec!["标签2".to_string(), "标签3".to_string()];

        let mut merged = current.clone();
        for tag in &new_tags {
            if !merged.contains(tag) {
                merged.push(tag.clone());
            }
        }

        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"标签1".to_string()));
        assert!(merged.contains(&"标签2".to_string()));
        assert!(merged.contains(&"标签3".to_string()));
    }

    #[test]
    fn test_tag_operation_remove_logic() {
        let mut current = vec![
            "标签1".to_string(),
            "标签2".to_string(),
            "标签3".to_string(),
        ];
        let remove_tags = ["标签2".to_string()];

        current.retain(|tag| !remove_tags.contains(tag));

        assert_eq!(current.len(), 2);
        assert!(!current.contains(&"标签2".to_string()));
        assert!(current.contains(&"标签1".to_string()));
        assert!(current.contains(&"标签3".to_string()));
    }

    #[test]
    fn test_accuracy_data_point_serialization() {
        let point = AccuracyDataPoint {
            date: "2026-04-28".to_string(),
            total: 100,
            correct: 85,
            accuracy: 0.85,
        };

        let json = serde_json::to_string(&point).unwrap();
        let deserialized: AccuracyDataPoint = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.date, "2026-04-28");
        assert_eq!(deserialized.total, 100);
        assert_eq!(deserialized.correct, 85);
        assert!((deserialized.accuracy - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_category_accuracy_serialization() {
        let cat = CategoryAccuracy {
            category: "风景".to_string(),
            total: 50,
            verified: 40,
            provisional: 8,
            rejected: 2,
            average_confidence: 0.82,
        };

        let json = serde_json::to_string(&cat).unwrap();
        let deserialized: CategoryAccuracy = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.category, "风景");
        assert_eq!(deserialized.total, 50);
        assert_eq!(deserialized.verified, 40);
        assert_eq!(deserialized.provisional, 8);
        assert_eq!(deserialized.rejected, 2);
        assert!((deserialized.average_confidence - 0.82).abs() < 0.001);
    }

    #[test]
    fn test_calibration_comparison_serialization() {
        let comp = CalibrationComparison {
            before_ece: 0.15,
            after_ece: 0.05,
            improvement_percent: 66.67,
        };

        let json = serde_json::to_string(&comp).unwrap();
        let deserialized: CalibrationComparison = serde_json::from_str(&json).unwrap();

        assert!((deserialized.before_ece - 0.15).abs() < 0.001);
        assert!((deserialized.after_ece - 0.05).abs() < 0.001);
        assert!((deserialized.improvement_percent - 66.67).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_trend_struct() {
        let trend = AccuracyTrend {
            daily_data: vec![
                AccuracyDataPoint {
                    date: "2026-04-01".to_string(),
                    total: 10,
                    correct: 8,
                    accuracy: 0.8,
                },
                AccuracyDataPoint {
                    date: "2026-04-02".to_string(),
                    total: 15,
                    correct: 13,
                    accuracy: 0.867,
                },
            ],
            category_accuracy: vec![CategoryAccuracy {
                category: "风景".to_string(),
                total: 50,
                verified: 40,
                provisional: 8,
                rejected: 2,
                average_confidence: 0.82,
            }],
            calibration_comparison: Some(CalibrationComparison {
                before_ece: 0.15,
                after_ece: 0.05,
                improvement_percent: 66.67,
            }),
        };

        let json = serde_json::to_string(&trend).unwrap();
        let deserialized: AccuracyTrend = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.daily_data.len(), 2);
        assert_eq!(deserialized.category_accuracy.len(), 1);
        assert!(deserialized.calibration_comparison.is_some());
    }

    #[test]
    fn test_accuracy_trend_null_comparison() {
        let trend = AccuracyTrend {
            daily_data: vec![],
            category_accuracy: vec![],
            calibration_comparison: None,
        };

        let json = serde_json::to_string(&trend).unwrap();
        let deserialized: AccuracyTrend = serde_json::from_str(&json).unwrap();

        assert!(deserialized.daily_data.is_empty());
        assert!(deserialized.calibration_comparison.is_none());
    }
}
