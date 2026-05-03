use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;
use std::sync::Arc;
use crate::core::ai_queue::AITaskQueue;
use crate::core::db::Database;
use crate::utils::error::AppResult;
use rusqlite::params;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIStatus {
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub retrying: usize,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResult {
    pub id: i64,
    pub file_name: String,
    pub ai_status: String,
    pub ai_tags: Option<String>,
    pub ai_description: Option<String>,
    pub ai_category: Option<String>,
    pub ai_error_message: Option<String>,
    pub ai_processed_at: Option<String>,
}

#[tauri::command]
pub async fn start_ai_processing(
    queue: State<'_, Arc<AITaskQueue>>,
) -> AppResult<AIStatus> {
    info!("启动 AI 处理队列");

    queue.start();
    queue.spawn_workers().await;
    queue.submit_pending_tasks().await;

    let status = queue.get_status();

    Ok(AIStatus {
        status: "processing".to_string(),
        total: status.total_tasks,
        completed: status.processed_tasks,
        failed: status.failed_tasks,
        retrying: 0,
        eta_seconds: None,
    })
}

#[tauri::command]
pub async fn pause_ai_processing(
    queue: State<'_, AITaskQueue>,
) -> AppResult<()> {
    info!("暂停 AI 处理队列");
    queue.pause();
    Ok(())
}

#[tauri::command]
pub async fn resume_ai_processing(
    queue: State<'_, AITaskQueue>,
) -> AppResult<()> {
    info!("恢复 AI 处理队列");
    queue.resume();
    Ok(())
}

#[tauri::command]
pub async fn get_ai_status(
    queue: State<'_, AITaskQueue>,
) -> AppResult<AIStatus> {
    let status = queue.get_status();

    let status_str = if !status.is_running {
        "idle"
    } else if status.is_paused {
        "paused"
    } else if status.total_tasks == 0 {
        "idle"
    } else if status.pending_tasks == 0 {
        "completed"
    } else {
        "processing"
    };

    Ok(AIStatus {
        status: status_str.to_string(),
        total: status.total_tasks,
        completed: status.processed_tasks,
        failed: status.failed_tasks,
        retrying: 0,
        eta_seconds: None,
    })
}

#[tauri::command]
pub async fn retry_failed_ai(
    queue: State<'_, AITaskQueue>,
    _image_id: Option<i64>,
) -> AppResult<usize> {
    info!("重试失败的 AI 任务");
    queue.submit_failed_retry_tasks().await;
    let status = queue.get_status();
    let retry_count = status.failed_tasks;

    Ok(retry_count)
}

#[tauri::command]
pub async fn get_recent_ai_results(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> AppResult<Vec<AIResult>> {
    let limit = limit.unwrap_or(50);
    
    let conn = db.open_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, file_name, ai_status, ai_tags, ai_description, ai_category, 
         ai_error_message, ai_processed_at 
         FROM images 
         WHERE ai_status IN ('completed', 'failed') 
         AND ai_processed_at IS NOT NULL
         ORDER BY ai_processed_at DESC 
         LIMIT ?"
    )?;
    
    let rows = stmt.query_map(params![limit], |row| {
        Ok(AIResult {
            id: row.get(0)?,
            file_name: row.get(1)?,
            ai_status: row.get(2)?,
            ai_tags: row.get(3).ok(),
            ai_description: row.get(4).ok(),
            ai_category: row.get(5).ok(),
            ai_error_message: row.get(6).ok(),
            ai_processed_at: row.get(7).ok(),
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
    fn test_ai_status_serialization_idle() -> Result<(), Box<dyn std::error::Error>> {
        let status = AIStatus {
            status: "idle".to_string(),
            total: 0,
            completed: 0,
            failed: 0,
            retrying: 0,
            eta_seconds: None,
        };

        let json = serde_json::to_string(&status)?;
        assert!(json.contains("idle"));
        assert!(json.contains("total"));

        let deserialized: AIStatus = serde_json::from_str(&json)?;
        assert_eq!(deserialized.status, "idle");
        assert_eq!(deserialized.total, 0);
        assert!(deserialized.eta_seconds.is_none());
        Ok(())
    }

    #[test]
    fn test_ai_status_serialization_processing() -> Result<(), Box<dyn std::error::Error>> {
        let status = AIStatus {
            status: "processing".to_string(),
            total: 100,
            completed: 50,
            failed: 5,
            retrying: 2,
            eta_seconds: Some(120),
        };

        let json = serde_json::to_string(&status)?;
        let deserialized: AIStatus = serde_json::from_str(&json)?;

        assert_eq!(deserialized.status, "processing");
        assert_eq!(deserialized.total, 100);
        assert_eq!(deserialized.completed, 50);
        assert_eq!(deserialized.failed, 5);
        assert_eq!(deserialized.retrying, 2);
        assert_eq!(deserialized.eta_seconds, Some(120));
        Ok(())
    }

    #[test]
    fn test_ai_status_serialization_completed() -> Result<(), Box<dyn std::error::Error>> {
        let status = AIStatus {
            status: "completed".to_string(),
            total: 50,
            completed: 50,
            failed: 0,
            retrying: 0,
            eta_seconds: Some(0),
        };

        let deserialized: AIStatus = serde_json::from_str(
            &serde_json::to_string(&status)?
        )?;

        assert_eq!(deserialized.status, "completed");
        assert_eq!(deserialized.completed, deserialized.total);
        Ok(())
    }

    #[test]
    fn test_ai_status_serialization_paused() -> Result<(), Box<dyn std::error::Error>> {
        let status = AIStatus {
            status: "paused".to_string(),
            total: 200,
            completed: 75,
            failed: 10,
            retrying: 0,
            eta_seconds: None,
        };

        let json = serde_json::to_string(&status)?;
        assert!(json.contains("paused"));

        let deserialized: AIStatus = serde_json::from_str(&json)?;
        assert_eq!(deserialized.status, "paused");
        assert_eq!(deserialized.total, 200);
        assert_eq!(deserialized.completed, 75);
        Ok(())
    }
}
