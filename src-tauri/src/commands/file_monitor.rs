#![allow(missing_docs)]
use crate::core::file_watcher::FileWatcherService;
use crate::utils::error::{AppError, AppResult};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::AppHandle;

pub struct MonitorState {
    pub service: Arc<Mutex<FileWatcherService>>,
}

#[tauri::command]
pub fn start_file_monitor(
    _app: AppHandle,
    state: tauri::State<'_, MonitorState>,
    directory: String,
) -> AppResult<()> {
    let dir = PathBuf::from(&directory);
    if !dir.exists() {
        return Err(AppError::config(format!("目录不存在: {}", directory)));
    }

    let service = state.service.lock().unwrap();
    service.watch_directory(&dir).map_err(AppError::config)?;
    tracing::info!("文件监控已启动: {}", directory);
    Ok(())
}

#[tauri::command]
pub fn stop_file_monitor(state: tauri::State<'_, MonitorState>) -> AppResult<()> {
    let service = state.service.lock().unwrap();
    service.unwatch();
    tracing::info!("文件监控已停止");
    Ok(())
}

#[tauri::command]
pub fn get_monitor_status(state: tauri::State<'_, MonitorState>) -> serde_json::Value {
    let service = state.service.lock().unwrap();
    serde_json::json!({
        "is_running": service.get_watched_count() > 0,
        "watched_directories": service.get_watched_count(),
    })
}
