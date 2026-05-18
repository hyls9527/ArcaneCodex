#![allow(missing_docs)]
use crate::core::db::Database;
use crate::core::xmp_service::{XmpMetadata, XmpService};
use crate::utils::error::{AppError, AppResult};
use tauri::State;

/// 读取文件的 XMP 元数据
#[tauri::command]
pub fn read_xmp_metadata(file_path: String) -> AppResult<serde_json::Value> {
    let path = std::path::Path::new(&file_path);
    match XmpService::read_xmp_from_file(path) {
        Ok(Some(meta)) => {
            let value = serde_json::to_value(meta).map_err(|e| AppError::config(e.to_string()))?;
            Ok(value)
        }
        Ok(None) => Ok(serde_json::json!({ "error": "no_xmp" })),
        Err(e) => Err(AppError::config(e)),
    }
}

/// 将 XMP 元数据写入文件
#[tauri::command]
pub fn write_xmp_metadata(file_path: String, metadata: serde_json::Value) -> AppResult<()> {
    let path = std::path::Path::new(&file_path);
    let meta: XmpMetadata =
        serde_json::from_value(metadata).map_err(|e| AppError::config(e.to_string()))?;

    XmpService::write_xmp_to_file(path, &meta).map_err(AppError::config)?;

    Ok(())
}

/// 生成 XMP Sidecar 文件
#[tauri::command]
pub fn generate_xmp_sidecar(image_path: String, metadata: serde_json::Value) -> AppResult<String> {
    let path = std::path::Path::new(&image_path);
    let meta: XmpMetadata =
        serde_json::from_value(metadata).map_err(|e| AppError::config(e.to_string()))?;

    let sidecar_path = XmpService::create_xmp_sidecar(path, &meta).map_err(AppError::config)?;

    Ok(sidecar_path.to_string_lossy().to_string())
}

/// 批量导出图片为 XMP Sidecar 文件
#[tauri::command]
pub fn export_as_xmp(image_ids: Vec<i64>, db: State<'_, Database>) -> AppResult<Vec<String>> {
    let conn = db.open_connection().map_err(AppError::database)?;
    let mut sidecar_paths = Vec::new();

    for chunk in image_ids.chunks(500) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, file_path FROM images WHERE id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(AppError::database)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AppError::database)?;

        for row in rows {
            let (id, path_str) = row.map_err(AppError::database)?;
            let path = std::path::Path::new(&path_str);
            let default_meta = XmpMetadata::default();

            match XmpService::create_xmp_sidecar(path, &default_meta) {
                Ok(p) => sidecar_paths.push(p.to_string_lossy().to_string()),
                Err(e) => tracing::warn!("生成XMP Sidecar失败 (id={}): {}", id, e),
            }
        }
    }

    Ok(sidecar_paths)
}
