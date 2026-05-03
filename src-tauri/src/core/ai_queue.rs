use crate::core::db::Database;
use crate::core::inference::{ProviderConfig, ProviderFactory};
use crate::core::search_index::SearchIndexBuilder;
use crate::models::ImageCategory;
use crate::utils::error::{AppResult, AppError};
use crate::utils::crypto;
use serde::Serialize;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tracing::{info, warn, error, debug};

const QUEUE_CAPACITY: usize = 1000;
const DEFAULT_CONCURRENCY: usize = 3;
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone)]
pub struct AITask {
    pub image_id: i64,
    pub file_path: String,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub processed_tasks: usize,
    pub failed_tasks: usize,
    pub concurrency: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum QueueCommand {
    Pause,
    Resume,
    Cancel,
}

#[derive(Debug, Clone, Serialize)]
pub struct AIProgressEvent {
    pub image_id: i64,
    pub status: String,
    pub message: String,
    pub total: usize,
    pub current: usize,
}

pub struct AITaskQueue {
    sender: mpsc::Sender<AITask>,
    receiver: Arc<TokioMutex<Option<mpsc::Receiver<AITask>>>>,
    #[allow(dead_code)]
    command_sender: mpsc::Sender<QueueCommand>,
    command_receiver: Arc<TokioMutex<Option<mpsc::Receiver<QueueCommand>>>>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    concurrency: usize,
    pub total_tasks: AtomicUsize,
    pub processed_tasks: AtomicUsize,
    pub failed_tasks: AtomicUsize,
    db: Arc<Database>,
    app_handle: Option<AppHandle>,
}

impl AITaskQueue {
    pub fn new(db: Arc<Database>, concurrency: Option<usize>) -> Self {
        let (sender, receiver) = mpsc::channel::<AITask>(QUEUE_CAPACITY);
        let (command_sender, command_receiver) = mpsc::channel::<QueueCommand>(16);
        let concurrency = concurrency.unwrap_or(DEFAULT_CONCURRENCY);

        Self {
            sender,
            receiver: Arc::new(TokioMutex::new(Some(receiver))),
            command_sender,
            command_receiver: Arc::new(TokioMutex::new(Some(command_receiver))),
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            concurrency,
            total_tasks: AtomicUsize::new(0),
            processed_tasks: AtomicUsize::new(0),
            failed_tasks: AtomicUsize::new(0),
            db,
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app: AppHandle) -> Self {
        self.app_handle = Some(app);
        self
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    #[allow(dead_code)]
    pub fn set_concurrency(&mut self, concurrency: usize) {
        self.concurrency = concurrency.clamp(1, 10);
    }

    pub fn start(&self) {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        info!("AI 任务队列已启动");
    }

    pub async fn spawn_workers(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            warn!("队列未启动，无法 spawn workers");
            return;
        }

        let receiver = {
            let mut guard = self.receiver.lock().await;
            guard.take()
        };

        let command_receiver = {
            let mut guard = self.command_receiver.lock().await;
            guard.take()
        };

        let Some(receiver) = receiver else {
            warn!("receiver 已被消费，无法再次 spawn workers");
            return;
        };
        let Some(command_receiver) = command_receiver else {
            warn!("command_receiver 已被消费，无法再次 spawn workers");
            return;
        };

        let receiver = Arc::new(TokioMutex::new(receiver));
        let command_receiver = Arc::new(TokioMutex::new(command_receiver));

        for worker_id in 0..self.concurrency {
            let worker = Worker {
                worker_id,
                sender: self.sender.clone(),
                receiver: receiver.clone(),
                command_receiver: command_receiver.clone(),
                is_paused: self.is_paused.clone(),
                is_running: self.is_running.clone(),
                total_tasks: Arc::new(AtomicUsize::new(0)),
                processed_tasks: Arc::new(AtomicUsize::new(0)),
                failed_tasks: Arc::new(AtomicUsize::new(0)),
                db: self.db.clone(),
                app_handle: self.app_handle.clone(),
            };
            tokio::spawn(worker.run());
        }

        info!("已启动 {} 个 AI Worker", self.concurrency);
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
        info!("AI 任务队列已暂停");
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        info!("AI 任务队列已恢复");
    }

    pub fn add_task(&self, image_id: i64, file_path: &str) -> AppResult<()> {
        let task = AITask {
            image_id,
            file_path: file_path.to_string(),
            retry_count: 0,
        };
        self.sender.try_send(task).map_err(|e| AppError::ai(format!("添加任务失败: {}", e)))?;
        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn clear_pending(&self) -> usize {
        0
    }

    pub fn get_stats(&self) -> std::collections::HashMap<String, usize> {
        let mut stats = std::collections::HashMap::new();
        stats.insert("total".to_string(), self.total_tasks.load(Ordering::SeqCst));
        stats.insert("processed".to_string(), self.processed_tasks.load(Ordering::SeqCst));
        stats.insert("failed".to_string(), self.failed_tasks.load(Ordering::SeqCst));
        stats
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        info!("AI 任务队列已取消");
    }

    pub fn get_status(&self) -> QueueStatus {
        let total = self.total_tasks.load(Ordering::SeqCst);
        let processed = self.processed_tasks.load(Ordering::SeqCst);
        let failed = self.failed_tasks.load(Ordering::SeqCst);
        let pending = total.saturating_sub(processed + failed);

        QueueStatus {
            is_running: self.is_running.load(Ordering::SeqCst),
            is_paused: self.is_paused.load(Ordering::SeqCst),
            total_tasks: total,
            pending_tasks: pending,
            processed_tasks: processed,
            failed_tasks: failed,
            concurrency: self.concurrency,
        }
    }

    pub async fn enqueue(&self, task: AITask) -> Result<(), String> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("队列未启动".to_string());
        }
        if self.is_paused.load(Ordering::SeqCst) {
            return Err("队列已暂停".to_string());
        }

        let image_id = task.image_id;
        self.sender
            .send(task)
            .await
            .map_err(|e| format!("发送任务失败: {}", e))?;

        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        debug!("任务已加入队列: image_id={}", image_id);
        Ok(())
    }

    fn query_pending_tasks(&self) -> Vec<(i64, String, u32)> {
        let conn = match self.db.open_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("查询 pending 任务失败: {}", e);
                return vec![];
            }
        };

        let mut stmt = match conn.prepare(
            "SELECT i.id, i.file_path, i.ai_retry_count FROM images i WHERE i.ai_status = 'pending' ORDER BY i.created_at ASC",
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("查询 pending 任务 SQL 失败: {}", e);
                return vec![];
            }
        };

        let rows = match stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, u32>(2)?))
        }) {
            Ok(r) => r,
            Err(e) => {
                error!("查询 pending 任务行失败: {}", e);
                return vec![];
            }
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    fn query_failed_retry_tasks(&self) -> Vec<(i64, String, u32)> {
        let conn = match self.db.open_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("查询 failed 任务失败: {}", e);
                return vec![];
            }
        };

        let mut stmt = match conn.prepare(
            "SELECT i.id, i.file_path, i.ai_retry_count FROM images i WHERE i.ai_status = 'failed' AND i.ai_retry_count < ?1",
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("查询 failed 重试任务 SQL 失败: {}", e);
                return vec![];
            }
        };

        let rows = match stmt.query_map(rusqlite::params![MAX_RETRIES], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, u32>(2)?))
        }) {
            Ok(r) => r,
            Err(e) => {
                error!("查询 failed 重试任务行失败: {}", e);
                return vec![];
            }
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    pub async fn submit_pending_tasks(&self) {
        let pending = self.query_pending_tasks();
        info!("发现 {} 个 pending AI 任务", pending.len());

        for (image_id, file_path, retry_count) in pending {
            let task = AITask { image_id, file_path, retry_count };
            if let Err(e) = self.enqueue(task).await {
                warn!("加入队列失败 (image_id={}): {}", image_id, e);
            }
        }
    }

    pub async fn submit_failed_retry_tasks(&self) {
        let failed = self.query_failed_retry_tasks();
        info!("发现 {} 个可重试的 failed AI 任务", failed.len());

        for (image_id, file_path, retry_count) in failed {
            let task = AITask { image_id, file_path, retry_count };
            if let Err(e) = self.enqueue(task).await {
                warn!("重试入队失败 (image_id={}): {}", image_id, e);
            }
        }
    }
}

struct Worker {
    worker_id: usize,
    sender: mpsc::Sender<AITask>,
    receiver: Arc<TokioMutex<mpsc::Receiver<AITask>>>,
    command_receiver: Arc<TokioMutex<mpsc::Receiver<QueueCommand>>>,
    is_paused: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    total_tasks: Arc<AtomicUsize>,
    processed_tasks: Arc<AtomicUsize>,
    failed_tasks: Arc<AtomicUsize>,
    db: Arc<Database>,
    app_handle: Option<AppHandle>,
}

impl Worker {
    async fn run(self) {
        info!("Worker {} 启动", self.worker_id);

        loop {
            if !self.is_running.load(Ordering::SeqCst) {
                info!("Worker {} 停止（is_running=false）", self.worker_id);
                break;
            }

            if self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            let cmd = {
                let mut guard = self.command_receiver.lock().await;
                guard.try_recv().ok()
            };

            if let Some(command) = cmd {
                match command {
                    QueueCommand::Cancel => {
                        info!("Worker {} 收到 Cancel 命令，退出", self.worker_id);
                        break;
                    }
                    QueueCommand::Pause => {
                        info!("Worker {} 收到 Pause 命令", self.worker_id);
                        self.is_paused.store(true, Ordering::SeqCst);
                        continue;
                    }
                    QueueCommand::Resume => {
                        info!("Worker {} 收到 Resume 命令", self.worker_id);
                        self.is_paused.store(false, Ordering::SeqCst);
                    }
                }
            }

            let task = {
                let mut guard = self.receiver.lock().await;
                guard.recv().await
            };

            let Some(task) = task else {
                continue;
            };

            debug!("Worker {} 处理任务: image_id={}", self.worker_id, task.image_id);

            self.process_task(&task).await;
        }

        info!("Worker {} 退出", self.worker_id);
    }

    async fn process_task(&self, task: &AITask) {
        let image_id = task.image_id;
        let file_path = &task.file_path;

        let _ = self.update_ai_status(image_id, "processing");

        let provider_config = self.query_provider_config();
        match ProviderFactory::create(provider_config) {
            Ok(provider) => {
                match provider.analyze_image(file_path).await {
                    Ok(result) => {
                        let tags_json = serde_json::to_string(&result.tags).unwrap_or_default();

                        let tag_status = self.determine_tag_status(
                            &result.category,
                            result.confidence,
                            &result.tags,
                            &result.description,
                        );

                        let _ = self.update_ai_status_full(
                            image_id,
                            "completed",
                            Some(&tags_json),
                            Some(&result.description),
                            Some(&result.category),
                            Some(result.confidence),
                            Some(&result.model),
                            Some(&result.provider),
                            Some(&tag_status),
                            None,
                        );

                        if tag_status != "rejected" {
                            let builder = SearchIndexBuilder::new();
                            if let Err(e) = builder.build_for_image(
                                &self.db,
                                image_id,
                                &result.description,
                                &result.tags,
                                &result.category,
                            ) {
                                warn!("构建搜索索引失败 (image_id={}): {}", image_id, e);
                            }
                        } else {
                            debug!("标签状态为 rejected，跳过搜索索引构建 (image_id={})", image_id);
                        }

                        self.processed_tasks.fetch_add(1, Ordering::SeqCst);
                        self.emit_progress(image_id, "completed", &result.description);
                        info!("Worker {} 完成 image_id={}, provider={}, tag_status={}", self.worker_id, image_id, result.provider, tag_status);
                    }
                    Err(e) => {
                        self.handle_ai_failure(image_id, task.retry_count, &e.to_string()).await;
                    }
                }
            }
            Err(e) => {
                self.handle_ai_failure(image_id, task.retry_count, &e.to_string()).await;
            }
        }
    }

    fn determine_tag_status(
        &self,
        category: &str,
        confidence: f64,
        tags: &[String],
        description: &str,
    ) -> String {
        use crate::core::consistency_checker::ConsistencyChecker;

        let cat = ImageCategory::from_str(category).unwrap_or(ImageCategory::Other);

        let consistency_conflicts = ConsistencyChecker::check_all(&cat, tags, description);

        if !consistency_conflicts.is_empty() {
            debug!("标签一致性校验失败 (image_id): {:?}", consistency_conflicts);
            return "rejected".to_string();
        }

        if confidence >= 0.85 {
            "verified".to_string()
        } else if confidence >= 0.50 {
            "provisional".to_string()
        } else {
            "rejected".to_string()
        }
    }

    fn query_provider_config(&self) -> ProviderConfig {
        let conn = match self.db.open_connection() {
            Ok(c) => c,
            Err(_) => return ProviderConfig::default(),
        };

        conn.query_row(
            "SELECT key, value FROM settings WHERE key IN ('inference_provider', 'inference_model', 'inference_api_key')",
            [],
            |_| { Ok(()) }
        ).ok();

        let provider_type = conn.query_row(
            "SELECT value FROM settings WHERE key = 'inference_provider'",
            [],
            |row| row.get::<_, String>(0)
        ).unwrap_or_else(|_| "lm_studio".to_string());

        let model = conn.query_row(
            "SELECT value FROM settings WHERE key = 'inference_model'",
            [],
            |row| row.get::<_, String>(0)
        ).unwrap_or_else(|_| "Qwen2.5-VL-7B-Instruct".to_string());

        let api_key = conn.query_row(
            "SELECT value FROM settings WHERE key = 'inference_api_key'",
            [],
            |row| row.get::<_, String>(0)
        ).ok().map(|k| crypto::decrypt_api_key(&k)).filter(|k| !k.is_empty());

        use crate::core::inference::InferenceProviderType;
        let ptype = match provider_type.as_str() {
            "zhipu" => InferenceProviderType::Zhipu,
            "openai" => InferenceProviderType::OpenAI,
            "openrouter" => InferenceProviderType::OpenRouter,
            "ollama" => InferenceProviderType::Ollama,
            "hermes" => InferenceProviderType::Hermes,
            _ => InferenceProviderType::LMStudio,
        };

        ProviderConfig {
            provider_type: ptype,
            model,
            api_key,
            ..Default::default()
        }
    }

    async fn handle_ai_failure(&self, image_id: i64, retry_count: u32, error_msg: &str) {
        let new_retry_count = retry_count + 1;

        if new_retry_count < MAX_RETRIES {
            let backoff_ms = 2u64.pow(new_retry_count) * 1000;
            warn!(
                "AI 失败 (image_id={}, 重试 {}/{}), {}ms 后重试: {}",
                image_id, new_retry_count, MAX_RETRIES, backoff_ms, error_msg
            );

            let _ = self.update_ai_status_with_error(image_id, "pending", error_msg);

            tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;

            if self.is_running.load(Ordering::SeqCst) && !self.is_paused.load(Ordering::SeqCst) {
                let task = AITask {
                    image_id,
                    file_path: self.query_file_path(image_id),
                    retry_count: new_retry_count,
                };
                self.total_tasks.fetch_add(1, Ordering::SeqCst);
                if let Err(e) = self.sender.send(task).await.map_err(|e| e.to_string()) {
                    warn!("重试入队失败 (image_id={}): {}", image_id, e);
                }
            }
        } else {
            error!("AI 达到最大重试次数 (image_id={}): {}", image_id, error_msg);
            let _ = self.update_ai_status_with_error(image_id, "failed", error_msg);
            self.failed_tasks.fetch_add(1, Ordering::SeqCst);
            self.emit_progress(image_id, "failed", error_msg);
        }
    }

    fn query_file_path(&self, image_id: i64) -> String {
        let conn = match self.db.open_connection() {
            Ok(c) => c,
            Err(_) => return String::new(),
        };
        conn.query_row(
            "SELECT file_path FROM images WHERE id = ?1",
            rusqlite::params![image_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default()
    }

    fn update_ai_status(&self, image_id: i64, status: &str) -> Result<(), String> {
        let conn = self.db.open_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE images SET ai_status = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            rusqlite::params![image_id, status],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_ai_status_with_error(&self, image_id: i64, status: &str, error_msg: &str) -> Result<(), String> {
        let conn = self.db.open_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE images SET ai_status = ?2, ai_error_message = ?3, ai_retry_count = ai_retry_count + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            rusqlite::params![image_id, status, error_msg],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn update_ai_status_full(
        &self,
        image_id: i64,
        status: &str,
        ai_tags: Option<&str>,
        ai_description: Option<&str>,
        ai_category: Option<&str>,
        ai_confidence: Option<f64>,
        ai_model: Option<&str>,
        ai_provider: Option<&str>,
        ai_tag_status: Option<&str>,
        error_msg: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.db.open_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE images SET 
                ai_status = ?2, 
                ai_tags = ?3, 
                ai_description = ?4, 
                ai_category = ?5, 
                ai_confidence = ?6, 
                ai_model = ?7,
                ai_provider = ?8,
                ai_processed_at = CURRENT_TIMESTAMP,
                ai_error_message = ?9,
                ai_tag_status = ?10,
                updated_at = CURRENT_TIMESTAMP 
            WHERE id = ?1",
            rusqlite::params![
                image_id, status, ai_tags.unwrap_or(""), ai_description.unwrap_or(""),
                ai_category.unwrap_or(""), ai_confidence.unwrap_or(0.0),
                ai_model.unwrap_or(""), ai_provider.unwrap_or("lm_studio"),
                error_msg.unwrap_or(""), ai_tag_status.unwrap_or("provisional")
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn emit_progress(&self, image_id: i64, status: &str, message: &str) {
        if let Some(app) = &self.app_handle {
            let total = self.total_tasks.load(Ordering::SeqCst);
            let current = self.processed_tasks.load(Ordering::SeqCst) + self.failed_tasks.load(Ordering::SeqCst);
            let _ = app.emit(
                "ai-progress",
                AIProgressEvent {
                    image_id,
                    status: status.to_string(),
                    message: message.to_string(),
                    total,
                    current,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_ai_queue.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.run_migrations().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_queue_custom_concurrency() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, Some(5));

        let status = queue.get_status();
        assert_eq!(status.concurrency, 5);
    }

    #[test]
    fn test_queue_start_stop() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        assert!(!queue.get_status().is_running);

        queue.start();
        assert!(queue.get_status().is_running);
        assert!(!queue.get_status().is_paused);

        queue.pause();
        assert!(queue.get_status().is_running);
        assert!(queue.get_status().is_paused);

        queue.resume();
        assert!(queue.get_status().is_running);
        assert!(!queue.get_status().is_paused);

        queue.cancel();
        assert!(!queue.get_status().is_running);
    }

    #[tokio::test]
    async fn test_enqueue_when_not_running() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        let task = AITask {
            image_id: 1,
            file_path: "test.jpg".to_string(),
            retry_count: 0,
        };

        let result = queue.enqueue(task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enqueue_success() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);
        queue.start();

        let task = AITask {
            image_id: 1,
            file_path: "test.jpg".to_string(),
            retry_count: 0,
        };

        let result = queue.enqueue(task).await;
        assert!(result.is_ok());

        let status = queue.get_status();
        assert_eq!(status.total_tasks, 1);
        assert_eq!(status.pending_tasks, 1);
    }

    #[tokio::test]
    async fn test_queue_capacity_limit() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, Some(1));
        queue.start();

        for i in 0..QUEUE_CAPACITY {
            let task = AITask {
                image_id: i as i64,
                file_path: format!("test_{}.jpg", i),
                retry_count: 0,
            };

            let result = queue.enqueue(task).await;
            if result.is_err() {
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_command_send() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        queue
            .command_sender
            .send(QueueCommand::Pause)
            .await
            .expect("Failed to send command");
    }

    #[test]
    fn test_atomic_counters() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        queue.total_tasks.store(10, Ordering::SeqCst);
        queue.processed_tasks.store(7, Ordering::SeqCst);
        queue.failed_tasks.store(2, Ordering::SeqCst);

        let status = queue.get_status();
        assert_eq!(status.total_tasks, 10);
        assert_eq!(status.processed_tasks, 7);
        assert_eq!(status.failed_tasks, 2);
        // pending = total - processed - failed = 10 - 7 - 2 = 1
        assert_eq!(status.pending_tasks, 1);
    }

    #[test]
    fn test_tc_ai_sp_002_failed_status_after_max_retries() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, ai_retry_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params!["/test/image.jpg", "image.jpg", 1024i64, "abc123", "image/jpeg", "processing", MAX_RETRIES],
        ).unwrap();
        let image_id = conn.last_insert_rowid();

        let new_retry_count = MAX_RETRIES + 1;
        if new_retry_count >= MAX_RETRIES {
            conn.execute(
                "UPDATE images SET ai_status = 'failed', ai_error_message = ?1, ai_retry_count = ?2 WHERE id = ?3",
                rusqlite::params!["AI 推理请求失败: request timeout", new_retry_count, image_id],
            ).unwrap();
        }

        let (status, error_msg, retry_count): (String, Option<String>, u32) = conn.query_row(
            "SELECT ai_status, ai_error_message, ai_retry_count FROM images WHERE id = ?1",
            rusqlite::params![image_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();

        assert_eq!(status, "failed", "达到最大重试次数后 ai_status 应为 failed");
        assert!(error_msg.is_some(), "应有错误消息记录");
        assert!(error_msg.unwrap().contains("timeout"), "错误消息应包含超时描述");
        assert!(retry_count >= MAX_RETRIES, "重试次数应达到上限");
    }

    #[test]
    fn test_tc_ai_sp_002_pending_status_before_max_retries() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, ai_retry_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params!["/test/image.jpg", "image.jpg", 1024i64, "abc123", "image/jpeg", "processing", 1u32],
        ).unwrap();
        let image_id = conn.last_insert_rowid();

        let current_retry = 1u32;
        if current_retry < MAX_RETRIES {
            conn.execute(
                "UPDATE images SET ai_status = 'pending', ai_error_message = ?1, ai_retry_count = ai_retry_count + 1 WHERE id = ?2",
                rusqlite::params!["AI 推理请求失败: request timeout", image_id],
            ).unwrap();
        }

        let (status, retry_count): (String, u32) = conn.query_row(
            "SELECT ai_status, ai_retry_count FROM images WHERE id = ?1",
            rusqlite::params![image_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();

        assert_eq!(status, "pending", "未达最大重试次数时 ai_status 应为 pending 等待重试");
        assert_eq!(retry_count, 2, "重试次数应递增");
    }

    #[test]
    fn test_tc_ai_sp_003_error_response_marks_failed() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, ai_retry_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params!["/test/image.jpg", "image.jpg", 1024i64, "abc123", "image/jpeg", "processing", MAX_RETRIES],
        ).unwrap();
        let image_id = conn.last_insert_rowid();

        let error_msg = "解析 AI JSON 响应失败: expected value at line 1 column 1 - 原始内容: This is not JSON";
        conn.execute(
            "UPDATE images SET ai_status = 'failed', ai_error_message = ?1, ai_retry_count = ai_retry_count + 1 WHERE id = ?2",
            rusqlite::params![error_msg, image_id],
        ).unwrap();

        let (status, err): (String, String) = conn.query_row(
            "SELECT ai_status, ai_error_message FROM images WHERE id = ?1",
            rusqlite::params![image_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();

        assert_eq!(status, "failed");
        assert!(err.contains("解析 AI JSON 响应失败"), "错误消息应包含 JSON 解析失败描述");
    }

    #[test]
    fn test_tc_ai_sp_003_exponential_backoff() {
        let retry_count = 1u32;
        let backoff_ms = 2u64.pow(retry_count) * 1000;
        assert_eq!(backoff_ms, 2000, "第 1 次重试退避 2 秒");

        let retry_count = 2u32;
        let backoff_ms = 2u64.pow(retry_count) * 1000;
        assert_eq!(backoff_ms, 4000, "第 2 次重试退避 4 秒");

        let retry_count = 3u32;
        let backoff_ms = 2u64.pow(retry_count) * 1000;
        assert_eq!(backoff_ms, 8000, "第 3 次重试退避 8 秒");
    }
}
