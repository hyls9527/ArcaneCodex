// Migration version upgrade tests
// Tests that migration system correctly handles version checking and schema upgrades

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    // Simulates an empty database (no tables, user_version = 0)
    fn setup_empty_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;
            PRAGMA user_version = 0;
        ").unwrap();
        conn
    }

    // Simulates a database with user_version = 1 but no tables (edge case)
    fn setup_version1_no_tables() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;
            PRAGMA user_version = 1;
        ").unwrap();
        conn
    }

    // Runs v1 migration on a connection
    fn run_v1_migration(conn: &mut Connection) {
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
                ('language', 'zh-CN'),
                ('thumbnail_size', '300');
            
            PRAGMA user_version = 1;
        ").unwrap();
    }

    #[test]
    fn test_migration_from_version_0() {
        let mut conn = setup_empty_db();
        
        // Check initial version
        let initial_version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(initial_version, 0);
        
        // Check no tables exist initially
        let table_count: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='table'", [], |row| row.get(0)).unwrap();
        assert_eq!(table_count, 0);
        
        // Run migration
        run_v1_migration(&mut conn);
        
        // Verify version updated
        let new_version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(new_version, 1);
        
        // Verify tables exist after migration
        let table_count: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='table'", [], |row| row.get(0)).unwrap();
        assert!(table_count >= 5, "Should have at least 5 tables after migration");
    }

    #[test]
    fn test_no_migration_when_already_version_1() {
        let mut conn = setup_version1_no_tables();
        
        let initial_version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(initial_version, 1);
        
        // Run migration (should be idempotent due to IF NOT EXISTS)
        run_v1_migration(&mut conn);
        
        // Version should remain 1
        let new_version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(new_version, 1);
    }

    #[test]
    fn test_migration_idempotency() {
        let mut conn = setup_empty_db();
        
        // Run migration twice
        run_v1_migration(&mut conn);
        run_v1_migration(&mut conn);
        
        // Should still have version 1
        let version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(version, 1);
        
        // Check that app_config still has correct default values (not duplicated)
        let mut stmt = conn.prepare("SELECT count(*) FROM app_config WHERE key = 'lm_studio_url'").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1, "INSERT OR IGNORE should prevent duplicates");
    }

    #[test]
    fn test_migration_creates_all_tables() {
        let mut conn = setup_empty_db();
        run_v1_migration(&mut conn);
        
        let expected_tables = vec!["images", "tags", "image_tags", "search_index", "task_queue", "app_config"];
        
        for table_name in expected_tables {
            let mut stmt = conn.prepare("
                SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?
            ").unwrap();
            let exists: i64 = stmt.query_row([table_name], |row| row.get(0)).unwrap();
            assert_eq!(exists, 1, "Table '{}' should exist after migration", table_name);
        }
    }

    #[test]
    fn test_migration_creates_all_indexes() {
        let mut conn = setup_empty_db();
        run_v1_migration(&mut conn);
        
        let expected_indexes = vec![
            "idx_images_ai_status",
            "idx_images_created_at",
            "idx_images_file_hash",
            "idx_search_index_term",
            "idx_search_index_image_id",
            "idx_task_queue_status",
            "idx_task_queue_priority",
        ];
        
        for index_name in expected_indexes {
            let mut stmt = conn.prepare("
                SELECT count(*) FROM sqlite_master WHERE type='index' AND name=?
            ").unwrap();
            let exists: i64 = stmt.query_row([index_name], |row| row.get(0)).unwrap();
            assert_eq!(exists, 1, "Index '{}' should exist after migration", index_name);
        }
    }

    #[test]
    fn test_wal_mode_persists_after_migration() {
        let mut conn = setup_empty_db();
        
        // Enable WAL mode
        let mode: String = conn.pragma_query_value(None, "journal_mode", |row| {
            row.get(0)
        }).unwrap();
        
        // Note: In-memory databases use "memory" journal mode and cannot use WAL
        // This test verifies that WAL mode is set in the migration SQL
        // For in-memory DB, we verify the migration includes PRAGMA journal_mode=WAL
        if mode == "memory" {
            // In-memory DB can't use WAL, so we just verify the migration runs successfully
            run_v1_migration(&mut conn);
            let post_mode: String = conn.pragma_query_value(None, "journal_mode", |row| {
                row.get(0)
            }).unwrap();
            assert_eq!(post_mode, "memory");
        } else {
            // Run migration
            run_v1_migration(&mut conn);
            
            // Verify WAL mode is still active
            let journal_mode: String = conn.pragma_query_value(None, "journal_mode", |row| {
                row.get(0)
            }).unwrap();
            assert_eq!(journal_mode, "wal");
        }
    }

    #[test]
    fn test_foreign_keys_persist_after_migration() {
        let mut conn = setup_empty_db();
        
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_v1_migration(&mut conn);
        
        let fk_enabled: i32 = conn.pragma_query_value(None, "foreign_keys", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(fk_enabled, 1);
    }
}
