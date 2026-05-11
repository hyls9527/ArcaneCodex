use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use tracing::info;

const SENSITIVE_DIRS: &[&str] = &[
    "C:\\Windows",
    "C:\\Program Files",
    "C:\\Program Files (x86)",
    "C:\\ProgramData",
    "/usr",
    "/bin",
    "/sbin",
    "/etc",
    "/var",
    "/System",
    "/Library",
];

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub format: String, // "json" 或 "csv"
    pub output_path: String,
    pub image_ids: Option<Vec<i64>>, // None = 导出所有
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub exported_count: usize,
    pub output_file: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct ImageMetadata {
    pub id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub width: i64,
    pub height: i64,
    pub file_hash: String,
    pub category: String,
    pub ai_description: String,
    pub ai_tags: String,
    pub ai_confidence: f64,
    pub ai_model: String,
    pub imported_at: String,
    pub ai_processed_at: String,
    pub exif_data: String,
    pub thumbnail_path: String,
}

#[tauri::command]
pub fn export_data(db: State<'_, Database>, request: ExportRequest) -> AppResult<ExportResult> {
    info!(
        "开始导出数据: format={}, path={}",
        request.format, request.output_path
    );

    let output_path = PathBuf::from(&request.output_path);

    validate_export_path(&output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::validation(format!("创建输出目录失败: {}", e)))?;
    }

    let metadata_list = fetch_image_metadata(&db, &request)?;

    let exported_count = match request.format.as_str() {
        "json" => export_to_json(&metadata_list, &output_path)?,
        "csv" => export_to_csv(&metadata_list, &output_path)?,
        _ => {
            return Err(AppError::validation(format!(
                "不支持的导出格式: {}。支持的格式: json, csv",
                request.format
            )))
        }
    };

    info!(
        "导出完成: {} 条记录 -> {}",
        exported_count,
        output_path.display()
    );

    Ok(ExportResult {
        exported_count,
        output_file: output_path.to_string_lossy().to_string(),
        format: request.format,
    })
}

fn fetch_image_metadata(db: &Database, request: &ExportRequest) -> AppResult<Vec<ImageMetadata>> {
    let conn = db.open_connection()?;

    let mut metadata_list = Vec::new();

    match &request.image_ids {
        Some(ids) => {
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
            let query = format!(
                "SELECT id, file_name, file_path, file_size, width, height, file_hash, \
                 category, ai_description, ai_tags, ai_confidence, ai_model, \
                 imported_at, ai_processed_at, exif_data, thumbnail_path \
                 FROM images WHERE id IN ({}) ORDER BY id",
                placeholders.join(",")
            );

            let mut stmt = conn.prepare(&query)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = ids
                .iter()
                .map(|id| id as &dyn rusqlite::types::ToSql)
                .collect();

            let rows = stmt.query_map(params.as_slice(), map_row_to_metadata)?;

            for row in rows {
                metadata_list.push(row?);
            }
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, file_name, file_path, file_size, width, height, file_hash, \
                 category, ai_description, ai_tags, ai_confidence, ai_model, \
                 imported_at, ai_processed_at, exif_data, thumbnail_path \
                 FROM images ORDER BY id",
            )?;

            let rows = stmt.query_map([], map_row_to_metadata)?;

            for row in rows {
                metadata_list.push(row?);
            }
        }
    }

    Ok(metadata_list)
}

fn map_row_to_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImageMetadata> {
    Ok(ImageMetadata {
        id: row.get(0)?,
        file_name: row.get(1)?,
        file_path: row.get(2)?,
        file_size: row.get(3)?,
        width: row.get(4)?,
        height: row.get(5)?,
        file_hash: row.get(6)?,
        category: row.get(7).unwrap_or_default(),
        ai_description: row.get(8).unwrap_or_default(),
        ai_tags: row.get(9).unwrap_or_default(),
        ai_confidence: row.get(10).unwrap_or(0.0),
        ai_model: row.get(11).unwrap_or_default(),
        imported_at: row.get(12)?,
        ai_processed_at: row.get(13).unwrap_or_default(),
        exif_data: row.get(14).unwrap_or_default(),
        thumbnail_path: row.get(15).unwrap_or_default(),
    })
}

fn export_to_json(metadata_list: &[ImageMetadata], output_path: &PathBuf) -> AppResult<usize> {
    let json = serde_json::to_string_pretty(metadata_list)
        .map_err(|e| AppError::validation(format!("序列化为 JSON 失败: {}", e)))?;

    fs::write(output_path, json)
        .map_err(|e| AppError::validation(format!("写入 JSON 文件失败: {}", e)))?;

    Ok(metadata_list.len())
}

fn export_to_csv(metadata_list: &[ImageMetadata], output_path: &PathBuf) -> AppResult<usize> {
    let mut csv_content = String::new();

    csv_content.push_str("id,file_name,file_path,file_size,width,height,file_hash,");
    csv_content.push_str("category,ai_description,ai_tags,ai_confidence,ai_model,");
    csv_content.push_str("imported_at,ai_processed_at,exif_data,thumbnail_path\n");

    for meta in metadata_list {
        csv_content.push_str(&format!(
            "{},\"{}\",\"{}\",{},{},{},\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
            meta.id,
            escape_csv(&meta.file_name),
            escape_csv(&meta.file_path),
            meta.file_size,
            meta.width,
            meta.height,
            escape_csv(&meta.file_hash),
            escape_csv(&meta.category),
            escape_csv(&meta.ai_description),
            escape_csv(&meta.ai_tags),
            meta.ai_confidence,
            escape_csv(&meta.ai_model),
            escape_csv(&meta.imported_at),
            escape_csv(&meta.ai_processed_at),
            escape_csv(&meta.exif_data),
            escape_csv(&meta.thumbnail_path),
        ));
    }

    fs::write(output_path, csv_content)
        .map_err(|e| AppError::validation(format!("写入 CSV 文件失败: {}", e)))?;

    Ok(metadata_list.len())
}

fn escape_csv(value: &str) -> String {
    value.replace("\"", "\"\"")
}

fn validate_export_path(path: &Path) -> AppResult<()> {
    let path_str = path.to_string_lossy().to_lowercase();

    for sensitive_dir in SENSITIVE_DIRS {
        let sensitive_lower = sensitive_dir.to_lowercase();
        if path_str.starts_with(&sensitive_lower) {
            return Err(AppError::validation(format!(
                "不允许导出到系统目录: {}",
                sensitive_dir
            )));
        }
    }

    if path.components().any(|c| c.as_os_str() == "..") {
        return Err(AppError::validation(
            "导出路径不能包含路径穿越 (..)".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_export.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE images (
                id INTEGER PRIMARY KEY,
                file_name TEXT,
                file_path TEXT,
                file_size INTEGER,
                width INTEGER,
                height INTEGER,
                file_hash TEXT,
                category TEXT,
                ai_description TEXT,
                ai_tags TEXT,
                ai_confidence REAL,
                ai_model TEXT,
                imported_at TEXT,
                ai_processed_at TEXT,
                exif_data TEXT,
                thumbnail_path TEXT
            );
        ",
        )
        .unwrap();

        (db, temp_dir)
    }

    #[test]
    fn test_export_request_deserialize() {
        let json = r#"{"format": "json", "output_path": "/tmp/export.json"}"#;
        let request: ExportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.format, "json");
        assert_eq!(request.output_path, "/tmp/export.json");
        assert!(request.image_ids.is_none());
    }

    #[test]
    fn test_export_request_with_image_ids() {
        let json = r#"{"format": "csv", "output_path": "/tmp/export.csv", "image_ids": [1, 2, 3]}"#;
        let request: ExportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.format, "csv");
        assert_eq!(request.image_ids, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_export_result_serialize() {
        let result = ExportResult {
            exported_count: 10,
            output_file: "/tmp/export.json".to_string(),
            format: "json".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("10"));
        assert!(json.contains("export.json"));
    }

    fn insert_test_image(db: &Database, id: i64) {
        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (id, file_name, file_path, file_size, width, height, file_hash, 
             category, ai_description, ai_tags, ai_confidence, ai_model, imported_at, 
             ai_processed_at, exif_data, thumbnail_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                id,
                format!("test{}.jpg", id),
                format!("C:/images/test{}.jpg", id),
                10000 + id * 1000,
                1920,
                1080,
                format!("hash{}", id),
                "风景",
                format!("一张测试的风景图片 #{}", id),
                "[\"风景\", \"自然\", \"测试\"]",
                0.95,
                "Qwen2.5-VL-7B",
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:01:00Z",
                "{}",
                format!("C:/thumbs/{}.webp", id),
            ],
        )
        .unwrap();
    }

    fn export_data_sync(db: &Database, request: ExportRequest) -> AppResult<ExportResult> {
        let output_path = PathBuf::from(&request.output_path);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::validation(format!("创建输出目录失败: {}", e)))?;
        }

        // Validate format first before querying database
        if !matches!(request.format.as_str(), "json" | "csv") {
            return Err(AppError::validation(format!(
                "不支持的导出格式: {}。支持的格式: json, csv",
                request.format
            )));
        }

        let metadata_list = fetch_image_metadata(db, &request)?;

        let exported_count = match request.format.as_str() {
            "json" => export_to_json(&metadata_list, &output_path)?,
            "csv" => export_to_csv(&metadata_list, &output_path)?,
            _ => unreachable!(),
        };

        Ok(ExportResult {
            exported_count,
            output_file: output_path.to_string_lossy().to_string(),
            format: request.format,
        })
    }

    #[test]
    fn test_export_to_json() {
        let (db, temp_dir) = setup_test_db();
        insert_test_image(&db, 1);

        let output_path = temp_dir.path().join("test.json");
        let request = ExportRequest {
            format: "json".to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            image_ids: None,
        };

        let result = export_data_sync(&db, request).unwrap();
        assert_eq!(result.exported_count, 1);
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("test1.jpg"));
        assert!(content.contains("风景"));
    }

    #[test]
    fn test_export_to_csv() {
        let (db, temp_dir) = setup_test_db();
        insert_test_image(&db, 1);
        insert_test_image(&db, 2);

        let output_path = temp_dir.path().join("test.csv");
        let request = ExportRequest {
            format: "csv".to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            image_ids: None,
        };

        let result = export_data_sync(&db, request).unwrap();
        assert_eq!(result.exported_count, 2);
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("id,file_name"));
        assert!(content.contains("test1.jpg"));
        assert!(content.contains("test2.jpg"));
    }

    #[test]
    fn test_export_with_image_ids() {
        let (db, temp_dir) = setup_test_db();
        insert_test_image(&db, 1);
        insert_test_image(&db, 2);
        insert_test_image(&db, 3);

        let output_path = temp_dir.path().join("filtered.json");
        let request = ExportRequest {
            format: "json".to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            image_ids: Some(vec![1, 3]),
        };

        let result = export_data_sync(&db, request).unwrap();
        assert_eq!(result.exported_count, 2);
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("test1.jpg"));
        assert!(content.contains("test3.jpg"));
        assert!(!content.contains("test2.jpg"));
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("has,comma"), "has,comma");
        assert_eq!(escape_csv("has\"quote"), "has\"\"quote");
        assert_eq!(escape_csv("has\nnewline"), "has\nnewline");
    }

    #[test]
    fn test_validate_export_path_rejects_system_dirs() {
        let windows_path = PathBuf::from("C:\\Windows\\System32\\export.json");
        assert!(validate_export_path(&windows_path).is_err());

        let program_files = PathBuf::from("C:\\Program Files\\app\\export.json");
        assert!(validate_export_path(&program_files).is_err());

        let etc_path = PathBuf::from("/etc/export.json");
        assert!(validate_export_path(&etc_path).is_err());

        let usr_path = PathBuf::from("/usr/local/export.json");
        assert!(validate_export_path(&usr_path).is_err());
    }

    #[test]
    fn test_validate_export_path_allows_user_dirs() {
        let user_path = PathBuf::from("C:\\Users\\test\\Documents\\export.json");
        assert!(validate_export_path(&user_path).is_ok());

        let home_path = PathBuf::from("/home/user/export.json");
        assert!(validate_export_path(&home_path).is_ok());

        let temp = TempDir::new().unwrap();
        let temp_path = temp.path().join("export.json");
        assert!(validate_export_path(&temp_path).is_ok());
    }

    #[test]
    fn test_validate_export_path_rejects_traversal() {
        let traversal_path = PathBuf::from("/home/user/../../../etc/export.json");
        assert!(validate_export_path(&traversal_path).is_err());
    }

    #[test]
    fn test_export_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let request = ExportRequest {
            format: "xml".to_string(),
            output_path: temp_dir
                .path()
                .join("test.xml")
                .to_string_lossy()
                .to_string(),
            image_ids: None,
        };

        let result = export_data_sync(&db, request);
        assert!(result.is_err());
        if let Err(e) = result {
            let err_str = e.to_string();
            assert!(
                err_str.contains("不支持的导出格式")
                    || err_str.contains("unsupported")
                    || err_str.contains("xml"),
                "Error should mention unsupported format, got: {}",
                err_str
            );
        }
    }
}
