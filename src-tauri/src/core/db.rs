use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tracing::{info, warn};

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type PooledConn = r2d2::PooledConnection<SqliteConnectionManager>;

#[derive(Clone)]
pub struct Database {
    pub db_path: Arc<PathBuf>,
    pool: SqlitePool,
}

impl Database {
    const PRAGMA_CONFIG: &'static str = "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        PRAGMA busy_timeout=5000;
    ";

    fn create_pool(db_path: &PathBuf) -> Result<SqlitePool> {
        let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
            conn.execute_batch(Self::PRAGMA_CONFIG)?;
            Ok(())
        });
        Pool::new(manager).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
    }

    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let db_path = get_db_path(app_handle);
        info!("Database path: {:?}", db_path);
        let pool = Self::create_pool(&db_path)?;
        Ok(Database {
            db_path: Arc::new(db_path),
            pool,
        })
    }

    #[cfg(test)]
    pub fn new_from_path(path: &str) -> Result<Self> {
        let db_path = PathBuf::from(path);
        let pool = Self::create_pool(&db_path)?;
        Ok(Database {
            db_path: Arc::new(db_path),
            pool,
        })
    }

    pub fn open_connection(&self) -> Result<PooledConn> {
        self.pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
    }

    #[allow(dead_code)]
    pub fn init(&self) -> Result<()> {
        self.run_migrations()
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.open_connection()?;
        info!("Running database migrations...");

        let user_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if user_version < 1 {
            info!("Applying migration v1: initial schema");
            drop(conn);
            self.apply_v1_initial_schema()?;
        }

        let conn = self.open_connection()?;
        let user_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if user_version < 2 {
            info!("Applying migration v2: comfyui generation support");
            drop(conn);
            self.apply_v2_comfyui_generation()?;
        }

        let conn = self.open_connection()?;
        let user_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        drop(conn);

        if user_version < 3 {
            info!("Applying migration v3: narrative anchor");
            self.apply_v3_narrative_anchor()?;
        }

        let conn = self.open_connection()?;
        let final_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        drop(conn);

        if final_version < 4 {
            info!("Applying migration v4: multi-provider inference support");
            self.apply_v4_multi_provider()?;
        }

        let conn = self.open_connection()?;
        let user_version_after_v4: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        drop(conn);

        if user_version_after_v4 < 5 {
            info!("Applying migration v5: AI tag status grading");
            self.apply_v5_ai_tag_status()?;
        }

        let conn = self.open_connection()?;
        let user_version_after_v5: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        drop(conn);

        if user_version_after_v5 < 6 {
            info!("Applying migration v6: unify config tables");
            self.apply_v6_unify_config()?;
        }

        let conn = self.open_connection()?;
        let user_version_after_v6: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        drop(conn);

        if user_version_after_v6 < 7 {
            info!("Applying migration v7: xmp sidecars support");
            self.apply_v7_xmp_sidecars()?;
        }

        let conn = self.open_connection()?;
        let final_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        info!("Database is up to date (version {})", final_version);

        Ok(())
    }

    fn apply_v1_initial_schema(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT UNIQUE NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_hash TEXT,
                mime_type TEXT,
                width INTEGER,
                height INTEGER,
                thumbnail_path TEXT,
                phash TEXT,
                exif_data JSON,
                ai_status TEXT NOT NULL DEFAULT 'pending',
                ai_tags JSON,
                ai_description TEXT,
                ai_category TEXT,
                ai_confidence REAL,
                ai_model TEXT,
                ai_processed_at DATETIME,
                ai_error_message TEXT,
                ai_retry_count INTEGER NOT NULL DEFAULT 0,
                source TEXT NOT NULL DEFAULT 'import',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_images_ai_status ON images(ai_status);
            CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_images_file_hash ON images(file_hash);

            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL COLLATE NOCASE,
                count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS image_tags (
                image_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS search_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                term TEXT NOT NULL,
                image_id INTEGER NOT NULL,
                field TEXT NOT NULL,
                position INTEGER,
                weight REAL NOT NULL DEFAULT 1.0,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_search_index_term ON search_index(term);
            CREATE INDEX IF NOT EXISTS idx_search_index_image_id ON search_index(image_id);

            CREATE TABLE IF NOT EXISTS task_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                task_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 0,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                error_message TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                started_at DATETIME,
                completed_at DATETIME,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_task_queue_status ON task_queue(status);
            CREATE INDEX IF NOT EXISTS idx_task_queue_priority ON task_queue(priority DESC, created_at ASC);

            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            INSERT OR IGNORE INTO app_config (key, value) VALUES
                ('lm_studio_url', 'http://localhost:1234'),
                ('ai_concurrency', '3'),
                ('ai_timeout_seconds', '60'),
                ('ai_max_retries', '3'),
                ('theme', 'system'),
                ('language', 'zh'),
                ('thumbnail_size', '300');

            PRAGMA user_version = 1;
        ")?;

        Ok(())
    }

    fn apply_v2_comfyui_generation(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            "
            ALTER TABLE images ADD COLUMN generation_source TEXT DEFAULT 'manual_import';
            ALTER TABLE images ADD COLUMN generation_metadata JSON;
            ALTER TABLE images ADD COLUMN generation_workflow_id TEXT;

            CREATE INDEX IF NOT EXISTS idx_images_generation_source ON images(generation_source);

            PRAGMA user_version = 2;
        ",
        )?;

        Ok(())
    }

    fn apply_v3_narrative_anchor(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS narratives (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                entities_json TEXT,
                embedding_json TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_narratives_image_id ON narratives(image_id);

            CREATE TABLE IF NOT EXISTS semantic_edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_narrative_id INTEGER NOT NULL,
                target_narrative_id INTEGER NOT NULL,
                similarity REAL NOT NULL,
                edge_type TEXT NOT NULL DEFAULT 'semantic',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (source_narrative_id) REFERENCES narratives(id) ON DELETE CASCADE,
                FOREIGN KEY (target_narrative_id) REFERENCES narratives(id) ON DELETE CASCADE,
                UNIQUE(source_narrative_id, target_narrative_id, edge_type)
            );

            CREATE INDEX IF NOT EXISTS idx_semantic_edges_source ON semantic_edges(source_narrative_id);
            CREATE INDEX IF NOT EXISTS idx_semantic_edges_target ON semantic_edges(target_narrative_id);

            PRAGMA user_version = 3;
        ")?;

        Ok(())
    }

    fn apply_v4_multi_provider(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            "
            -- 添加 ai_provider 字段到 images 表
            ALTER TABLE images ADD COLUMN ai_provider TEXT DEFAULT 'lm_studio';
            
            -- 创建 settings 表（如果不存在）
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 插入推理提供者相关配置
            INSERT OR REPLACE INTO settings (key, value) VALUES
                ('inference_provider', 'lm_studio'),
                ('inference_model', 'Qwen2.5-VL-7B-Instruct'),
                ('inference_api_key', ''),
                ('inference_timeout', '60');
            
            PRAGMA user_version = 4;
        ",
        )?;

        Ok(())
    }

    fn apply_v5_ai_tag_status(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch("
            -- 添加 ai_tag_status 字段到 images 表
            -- verified: 校准后高置信度，参与搜索
            -- provisional: 中置信度，标记待验证
            -- rejected: 低置信度，拒绝入库
            ALTER TABLE images ADD COLUMN ai_tag_status TEXT DEFAULT 'provisional';
            
            -- 为现有数据设置默认状态
            UPDATE images SET ai_tag_status = 'provisional' 
            WHERE ai_status = 'completed' AND ai_tag_status IS NULL;
            
            -- 为未处理的图片设置为 rejected (无 AI 分析)
            UPDATE images SET ai_tag_status = 'rejected' 
            WHERE ai_status = 'pending' AND ai_tag_status IS NULL;
            
            -- 为失败的处理设置为 rejected
            UPDATE images SET ai_tag_status = 'rejected' 
            WHERE ai_status = 'failed' AND ai_tag_status IS NULL;
            
            -- 创建校准样本表（用于收集人工标注数据）
            CREATE TABLE IF NOT EXISTS calibration_samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                predicted_category TEXT NOT NULL,
                raw_confidence REAL NOT NULL,
                is_correct BOOLEAN NOT NULL,
                annotated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (image_id) REFERENCES images(id)
            );
            
            -- 创建校准报告历史表
            CREATE TABLE IF NOT EXISTS calibration_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                report_json TEXT NOT NULL,
                total_samples INTEGER NOT NULL,
                overall_ece REAL NOT NULL,
                computed_at TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 创建校准曲线缓存表
            CREATE TABLE IF NOT EXISTS calibration_curves (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                curve_json TEXT NOT NULL,
                total_samples INTEGER NOT NULL,
                computed_at TEXT NOT NULL,
                UNIQUE(category)
            );
            
            -- 创建标签修正历史表
            CREATE TABLE IF NOT EXISTS tag_corrections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                old_tags JSON NOT NULL,
                new_tags JSON NOT NULL,
                corrected_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (image_id) REFERENCES images(id)
            );
            
            -- 创建错误模式库表
            CREATE TABLE IF NOT EXISTS error_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_name TEXT NOT NULL,
                pattern_description TEXT,
                occurrence_count INTEGER NOT NULL DEFAULT 1,
                first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(pattern_name)
            );
            
            -- 索引
            CREATE INDEX IF NOT EXISTS idx_images_ai_tag_status ON images(ai_tag_status);
            CREATE INDEX IF NOT EXISTS idx_cal_samples_category ON calibration_samples(predicted_category);
            CREATE INDEX IF NOT EXISTS idx_cal_samples_image_id ON calibration_samples(image_id);
            CREATE INDEX IF NOT EXISTS idx_tag_corrections_image_id ON tag_corrections(image_id);
            
            PRAGMA user_version = 5;
        ")?;

        Ok(())
    }

    fn apply_v6_unify_config(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            "
            INSERT OR IGNORE INTO settings (key, value)
                SELECT key, value FROM app_config;

            DROP TABLE IF EXISTS app_config;

            PRAGMA user_version = 6;
        ",
        )?;
        Ok(())
    }

    fn apply_v7_xmp_sidecars(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS xmp_sidecars (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
                sidecar_path TEXT UNIQUE NOT NULL,
                last_synced_at DATETIME,
                is_dirty INTEGER NOT NULL DEFAULT 0,
                hash TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_xmp_sidecars_image_id ON xmp_sidecars(image_id);
            CREATE INDEX IF NOT EXISTS idx_xmp_sidecars_dirty ON xmp_sidecars(is_dirty);

            PRAGMA user_version = 7;
        ",
        )?;
        Ok(())
    }
}

fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    let _ = std::fs::create_dir_all(&app_data);
    app_data.join("arcanecodex.db")
}

#[allow(dead_code)]
pub fn init_database(app_handle: &tauri::AppHandle) -> Result<()> {
    let db_path = get_db_path(app_handle);
    info!("Database path: {:?}", db_path);

    match try_open_database(&db_path) {
        Ok(_) => {
            let db = Database::new(app_handle)?;
            db.run_migrations()?;
            info!("Database initialized successfully");
        }
        Err(e) => {
            warn!("Database corrupted or invalid: {}", e);
            warn!("Attempting to recover by renaming corrupted database...");

            let backup_path = db_path.with_extension("db.corrupted");
            if db_path.exists() {
                std::fs::rename(&db_path, &backup_path).map_err(|rename_err| {
                    rusqlite::Error::InvalidParameterName(format!(
                        "无法重命名损坏的数据库文件: {}",
                        rename_err
                    ))
                })?;
                info!("Corrupted database renamed to: {:?}", backup_path);
            }

            let wal_path = db_path.with_extension("db-wal");
            if wal_path.exists() {
                let _ = std::fs::rename(&wal_path, backup_path.with_extension("db-wal.corrupted"));
            }
            let shm_path = db_path.with_extension("db-shm");
            if shm_path.exists() {
                let _ = std::fs::rename(&shm_path, backup_path.with_extension("db-shm.corrupted"));
            }

            let db = Database::new(app_handle)?;
            db.run_migrations()?;
            info!("Fresh database initialized after corruption recovery");
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn try_open_database(db_path: &PathBuf) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA integrity_check;")?;

    match conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0)) {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Cannot read user_version: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_lock.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_busy_timeout_is_configured() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000, "busy_timeout should be 5000ms");
    }

    #[test]
    fn test_concurrent_writes_succeed_with_busy_timeout() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('test_key', 'initial')",
            [],
        )
        .unwrap();
        drop(conn);

        let success1 = Arc::new(AtomicBool::new(false));
        let success2 = Arc::new(AtomicBool::new(false));

        let s1 = success1.clone();
        let s2 = success2.clone();
        let db_clone1 = db.clone();
        let db_clone2 = db.clone();

        let handle1 = thread::spawn(move || {
            let conn = db_clone1.open_connection().unwrap();
            conn.execute("BEGIN IMMEDIATE", []).unwrap();
            thread::sleep(Duration::from_millis(100));
            conn.execute(
                "UPDATE settings SET value = 'thread1' WHERE key = 'test_key'",
                [],
            )
            .unwrap();
            conn.execute("COMMIT", []).unwrap();
            s1.store(true, Ordering::SeqCst);
        });

        let handle2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            let conn = db_clone2.open_connection().unwrap();
            let result = conn.execute(
                "UPDATE settings SET value = 'thread2' WHERE key = 'test_key'",
                [],
            );
            if result.is_ok() {
                s2.store(true, Ordering::SeqCst);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert!(
            success1.load(Ordering::SeqCst) || success2.load(Ordering::SeqCst),
            "At least one concurrent write should succeed with busy_timeout"
        );

        let conn = db.open_connection().unwrap();
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'test_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            value == "thread1" || value == "thread2",
            "Final value should be from one of the threads"
        );
    }

    #[test]
    fn test_wal_mode_allows_concurrent_reads() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal", "journal_mode should be WAL");

        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('wal_test', 'data')",
            [],
        )
        .unwrap();
        drop(conn);

        let db1 = db.clone();
        let db2 = db.clone();

        let handle1 = thread::spawn(move || {
            let conn = db1.open_connection().unwrap();
            let value: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'wal_test'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(value, "data");
        });

        let handle2 = thread::spawn(move || {
            let conn = db2.open_connection().unwrap();
            let value: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'wal_test'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(value, "data");
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    #[test]
    fn test_corrupted_database_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_corrupt.db");

        std::fs::write(&db_path, [0u8; 1024]).unwrap();

        let result = try_open_database(&db_path);
        assert!(result.is_err(), "Corrupted database should fail to open");

        let _ = std::fs::remove_file(&db_path);
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();

        let conn = db.open_connection().unwrap();
        let timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }

    #[test]
    fn test_missing_database_creates_fresh() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("nonexistent.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();

        assert!(db_path.exists(), "Database file should be created");
        let conn = db.open_connection().unwrap();
        let timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }
}
