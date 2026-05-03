// Database schema validation tests
// Tests that all tables, indexes, and default config values are created correctly

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Enable WAL mode
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;
            PRAGMA busy_timeout=5000;
        ").unwrap();
        
        // Run v1 initial schema (copied from db.rs)
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
        
        conn
    }

    #[test]
    fn test_images_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='images'
        ").unwrap();
        let exists = stmt.exists([]).unwrap();
        assert!(exists, "images table should exist");
    }

    #[test]
    fn test_images_table_columns() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("PRAGMA table_info(images)").unwrap();
        let columns: Vec<String> = stmt.query_map([], |row| {
            row.get::<_, String>(1)
        }).unwrap().map(|r| r.unwrap()).collect();
        
        let expected_columns = vec![
            "id", "file_path", "file_name", "file_size", "file_hash",
            "mime_type", "width", "height", "thumbnail_path", "phash",
            "exif_data", "ai_status", "ai_tags", "ai_description",
            "ai_category", "ai_confidence", "ai_model", "ai_processed_at",
            "ai_error_message", "ai_retry_count", "source", "created_at", "updated_at"
        ];
        
        for col in expected_columns {
            assert!(columns.contains(&col.to_string()), "images table should have column: {}", col);
        }
    }

    #[test]
    fn test_tags_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='tags'
        ").unwrap();
        assert!(stmt.exists([]).unwrap(), "tags table should exist");
    }

    #[test]
    fn test_image_tags_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='image_tags'
        ").unwrap();
        assert!(stmt.exists([]).unwrap(), "image_tags table should exist");
    }

    #[test]
    fn test_search_index_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='search_index'
        ").unwrap();
        assert!(stmt.exists([]).unwrap(), "search_index table should exist");
    }

    #[test]
    fn test_task_queue_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='task_queue'
        ").unwrap();
        assert!(stmt.exists([]).unwrap(), "task_queue table should exist");
    }

    #[test]
    fn test_app_config_table_exists() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='app_config'
        ").unwrap();
        assert!(stmt.exists([]).unwrap(), "app_config table should exist");
    }

    #[test]
    fn test_images_indexes() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='index' AND tbl_name='images'
        ").unwrap();
        let indexes: Vec<String> = stmt.query_map([], |row| {
            row.get::<_, String>(0)
        }).unwrap().map(|r| r.unwrap()).collect();
        
        assert!(indexes.contains(&"idx_images_ai_status".to_string()));
        assert!(indexes.contains(&"idx_images_created_at".to_string()));
        assert!(indexes.contains(&"idx_images_file_hash".to_string()));
    }

    #[test]
    fn test_search_index_indexes() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='index' AND tbl_name='search_index'
        ").unwrap();
        let indexes: Vec<String> = stmt.query_map([], |row| {
            row.get::<_, String>(0)
        }).unwrap().map(|r| r.unwrap()).collect();
        
        assert!(indexes.contains(&"idx_search_index_term".to_string()));
        assert!(indexes.contains(&"idx_search_index_image_id".to_string()));
    }

    #[test]
    fn test_task_queue_indexes() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("
            SELECT name FROM sqlite_master 
            WHERE type='index' AND tbl_name='task_queue'
        ").unwrap();
        let indexes: Vec<String> = stmt.query_map([], |row| {
            row.get::<_, String>(0)
        }).unwrap().map(|r| r.unwrap()).collect();
        
        assert!(indexes.contains(&"idx_task_queue_status".to_string()));
        assert!(indexes.contains(&"idx_task_queue_priority".to_string()));
    }

    #[test]
    fn test_app_config_default_values() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("SELECT key, value FROM app_config").unwrap();
        let configs: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).unwrap().map(|r| r.unwrap()).collect();
        
        let config_map: std::collections::HashMap<String, String> = configs.into_iter().collect();
        
        assert_eq!(config_map.get("lm_studio_url").unwrap(), "http://localhost:1234");
        assert_eq!(config_map.get("ai_concurrency").unwrap(), "3");
        assert_eq!(config_map.get("ai_timeout_seconds").unwrap(), "60");
        assert_eq!(config_map.get("ai_max_retries").unwrap(), "3");
        assert_eq!(config_map.get("theme").unwrap(), "system");
        assert_eq!(config_map.get("language").unwrap(), "zh-CN");
        assert_eq!(config_map.get("thumbnail_size").unwrap(), "300");
    }

    #[test]
    fn test_user_version() {
        let conn = setup_test_db();
        let version: i32 = conn.pragma_query_value(None, "user_version", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(version, 1, "Database user_version should be 1");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = setup_test_db();
        let fk_enabled: i32 = conn.pragma_query_value(None, "foreign_keys", |row| {
            row.get(0)
        }).unwrap();
        assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
    }

    #[test]
    fn test_images_default_values() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size) VALUES (?, ?, ?)",
            rusqlite::params!["/test/image.jpg", "image.jpg", 1024]
        ).unwrap();
        
        let (ai_status, ai_retry_count, source): (String, i32, String) = conn.query_row(
            "SELECT ai_status, ai_retry_count, source FROM images WHERE file_path = ?",
            ["/test/image.jpg"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).unwrap();
        
        assert_eq!(ai_status, "pending");
        assert_eq!(ai_retry_count, 0);
        assert_eq!(source, "import");
    }

    #[test]
    fn test_tags_case_insensitive() {
        let conn = setup_test_db();
        conn.execute("INSERT INTO tags (name) VALUES (?)", ["nature"]).unwrap();
        
        // Try to insert same tag with different case - should fail due to UNIQUE COLLATE NOCASE
        let result = conn.execute("INSERT INTO tags (name) VALUES (?)", ["Nature"]);
        assert!(result.is_err(), "Tag names should be case-insensitive unique");
    }
}
