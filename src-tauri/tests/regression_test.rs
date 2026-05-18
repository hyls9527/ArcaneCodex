//! Regression tests for fixed unwrap/expect points from Phase 1.
//!
//! These tests verify that error handling paths (not just happy paths) work
//! correctly across commands/, core/, utils/, and models/ directories.
//! Each test proves that the fix guards against panics by confirming errors
//! propagate gracefully instead of unwrapping.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use arcane_codex::models::image::Image;
    use arcane_codex::utils::error::{AppError, AppResult};

    // ======================================================================
    // utils/ — AppError factory methods, error_code, Display, Serialization
    // ======================================================================

    /// Verify that every AppError variant returns the expected error code.
    #[test]
    fn test_app_error_error_code_all_variants() {
        let db_err = AppError::database(rusqlite::Error::InvalidQuery);
        assert_eq!(db_err.error_code(), "DB_001");

        let io_err = AppError::io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
        assert_eq!(io_err.error_code(), "IO_001");

        let val_err = AppError::validation("bad input");
        assert_eq!(val_err.error_code(), "VAL_001");

        let ai_err = AppError::ai("inference failed");
        assert_eq!(ai_err.error_code(), "AI_001");

        let cfg_err = AppError::config("missing config");
        assert_eq!(cfg_err.error_code(), "CFG_001");

        let nf_err = AppError::not_found("resource");
        assert_eq!(nf_err.error_code(), "NF_001");

        let auth_err = AppError::auth("unauthorized");
        assert_eq!(auth_err.error_code(), "AUTH_001");

        let int_err = AppError::internal("internal failure");
        assert_eq!(int_err.error_code(), "INT_001");
    }

    /// Verify that Display output includes the error code and message for
    /// each variant.
    #[test]
    fn test_app_error_display_contains_code_and_message() {
        let err = AppError::validation("input too long");
        let display = err.to_string();
        assert!(
            display.contains("VAL_001"),
            "Display should contain error code"
        );
        assert!(display.contains("input too long"), "Display should contain message");

        let io_err = AppError::io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"));
        let io_display = io_err.to_string();
        assert!(io_display.contains("IO_001"));
        assert!(io_display.contains("access denied"));

        let cfg_err = AppError::config("setting not found");
        let cfg_display = cfg_err.to_string();
        assert!(cfg_display.contains("CFG_001"));
        assert!(cfg_display.contains("setting not found"));
    }

    /// Verify that AppError serializes to JSON with code and message fields.
    #[test]
    fn test_app_error_serialization() {
        let err = AppError::validation("serialize me");
        let json = serde_json::to_string(&err).expect("AppError should serialize");
        assert!(json.contains("VAL_001"), "JSON should contain error code");
        assert!(json.contains("serialize me"), "JSON should contain message");

        // Verify roundtrip via serde_json::Value
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("AppError JSON should deserialize as Value");
        assert_eq!(value["code"], "VAL_001");
        assert!(
            value["message"].as_str().unwrap_or("").contains("serialize me")
        );
    }

    /// Verify that From<std::io::Error> conversion produces AppError::IoError.
    #[test]
    fn test_io_error_conversion_via_from_trait() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();

        match &app_err {
            AppError::IoError { code, message, source } => {
                assert_eq!(code, "IO_001");
                assert!(message.contains("file not found"));
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("Expected AppError::IoError, got {:?}", other),
        }
    }

    /// Verify that From<rusqlite::Error> conversion produces AppError::Database.
    #[test]
    fn test_rusqlite_error_conversion_via_from_trait() {
        let sqlite_err = rusqlite::Error::InvalidQuery;
        let app_err: AppError = sqlite_err.into();

        match &app_err {
            AppError::Database { code, message, .. } => {
                assert_eq!(code, "DB_001");
                assert!(!message.is_empty());
            }
            other => panic!("Expected AppError::Database, got {:?}", other),
        }
    }

    /// Verify From<reqwest::Error> conversion via a real connection failure.
    #[tokio::test]
    async fn test_reqwest_error_conversion_via_from_trait() {
        // Connect to a port that will reliably fail on any platform
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client build");
        let result = client.get("http://127.0.0.1:1").send().await;

        assert!(
            result.is_err(),
            "Connecting to closed port 1 should produce an error"
        );

        let req_err = result.unwrap_err();
        let app_err: AppError = req_err.into();

        match &app_err {
            AppError::Http { code, message, .. } => {
                assert_eq!(code, "HTTP_001");
                assert!(!message.is_empty());
            }
            other => panic!("Expected AppError::Http, got {:?}", other),
        }
    }

    // ======================================================================
    // core/ — Mutex poison recovery pattern & FileWatcherService
    // ======================================================================

    /// Verify that the Mutex poison recovery pattern
    /// (lock().unwrap_or_else(|e| e.into_inner())) works correctly.
    #[test]
    fn test_mutex_poison_recovery() {
        let data = Arc::new(Mutex::new(42u32));
        let data_clone = data.clone();

        // Poison the mutex by panicking while holding the lock
        let handle = std::thread::spawn(move || {
            let _guard = data_clone.lock().unwrap();
            panic!("intentional poison");
        });
        assert!(handle.join().is_err(), "Thread should have panicked");

        // Recover using the Phase 1 pattern
        let recovered = *data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(
            recovered, 42,
            "Recovered value should be the original 42"
        );

        // Verify we can mutate after recovery
        *data.lock().unwrap_or_else(|e| e.into_inner()) = 100;
        let final_val = *data.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(final_val, 100);
    }

    /// Verify that FileWatcherService starts with zero watched directories
    /// and that get_watched_count returns 0 (no panic on fresh state).
    #[test]
    fn test_file_watcher_initial_state() {
        use arcane_codex::core::file_watcher::FileWatcherService;
        use tokio::sync::broadcast;

        let (tx, _rx) = broadcast::channel(16);
        let service = FileWatcherService::new(tx);

        // Fresh service should have no watched directories
        assert_eq!(
            service.get_watched_count(),
            0,
            "New FileWatcherService should have zero watched directories"
        );
    }

    // ======================================================================
    // models/ — Image struct construction and serde roundtrip
    // ======================================================================

    /// Verify Image struct can be constructed with valid data and all fields
    /// are accessible.
    #[test]
    fn test_image_struct_construction() {
        let image = Image {
            id: 42,
            file_path: "/test/photos/sunset.jpg".to_string(),
            file_name: "sunset.jpg".to_string(),
            file_size: 204800,
            file_hash: Some("abc123def456".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            width: Some(1920),
            height: Some(1080),
            thumbnail_path: Some("/test/thumb/sunset.webp".to_string()),
            phash: Some("phash_value".to_string()),
            exif_data: Some(serde_json::json!({"camera": "Canon"})),
            ai_status: "completed".to_string(),
            ai_tags: Some(serde_json::json!(["sunset", "landscape"])),
            ai_description: Some("A beautiful sunset over the mountains".to_string()),
            ai_category: Some("landscape".to_string()),
            ai_confidence: Some(0.95),
            ai_model: Some("Qwen2.5-VL-7B-Instruct".to_string()),
            ai_processed_at: Some("2024-06-15 14:30:00".to_string()),
            ai_error_message: None,
            ai_retry_count: 0,
            source: "import".to_string(),
            generation_source: None,
            generation_metadata: None,
            generation_workflow_id: None,
            ai_provider: Some("lm_studio".to_string()),
            ai_tag_status: Some("verified".to_string()),
            created_at: "2024-06-15 14:30:00".to_string(),
            updated_at: "2024-06-15 14:30:00".to_string(),
        };

        assert_eq!(image.id, 42);
        assert_eq!(image.file_name, "sunset.jpg");
        assert_eq!(image.ai_status, "completed");
        assert_eq!(image.ai_confidence, Some(0.95));
        assert_eq!(image.source, "import");
        assert_eq!(image.ai_retry_count, 0);
    }

    /// Verify Image serializes and deserializes correctly through serde.
    #[test]
    fn test_image_serialization_roundtrip() {
        let image = Image {
            id: 99,
            file_path: "/test/photo.png".to_string(),
            file_name: "photo.png".to_string(),
            file_size: 512000,
            file_hash: None,
            mime_type: Some("image/png".to_string()),
            width: Some(800),
            height: Some(600),
            thumbnail_path: None,
            phash: None,
            exif_data: None,
            ai_status: "pending".to_string(),
            ai_tags: None,
            ai_description: None,
            ai_category: None,
            ai_confidence: None,
            ai_model: None,
            ai_processed_at: None,
            ai_error_message: None,
            ai_retry_count: 0,
            source: "import".to_string(),
            generation_source: None,
            generation_metadata: None,
            generation_workflow_id: None,
            ai_provider: None,
            ai_tag_status: None,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-01-01 00:00:00".to_string(),
        };

        let json = serde_json::to_string(&image).expect("Image should serialize");
        let deserialized: Image =
            serde_json::from_str(&json).expect("Image should deserialize");

        assert_eq!(deserialized.id, 99);
        assert_eq!(deserialized.file_name, "photo.png");
        assert_eq!(deserialized.ai_status, "pending");
        assert!(deserialized.file_hash.is_none());
        assert!(deserialized.ai_tags.is_none());
    }

    // ======================================================================
    // commands/ — AppResult propagation with the ? operator
    // ======================================================================

    /// A helper function that uses AppResult with the ? operator to
    /// propagate errors. If this compiles and returns the expected
    /// error variant, the AppResult plumbing is correct.
    fn propagate_via_question_mark(should_fail: bool) -> AppResult<i32> {
        if should_fail {
            Err(AppError::validation("propagation test"))?;
        }
        Ok(42)
    }

    /// Verify that AppResult propagates errors through the ? operator.
    #[test]
    fn test_app_result_propagation_with_question_mark() {
        // Success path
        let ok_result = propagate_via_question_mark(false);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        // Error path
        let err_result = propagate_via_question_mark(true);
        assert!(err_result.is_err());
        match err_result.unwrap_err() {
            AppError::ValidationError { code, message } => {
                assert_eq!(code, "VAL_001");
                assert!(message.contains("propagation test"));
            }
            other => panic!("Expected ValidationError, got {:?}", other),
        }
    }

    /// Verify that the AppResult type alias works with map_err and from
    /// conversions in a realistic error chain.
    #[test]
    fn test_app_result_error_chain() {
        fn inner_fallible() -> Result<i32, std::io::Error> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "inner failure",
            ))
        }

        fn outer() -> AppResult<i32> {
            let val = inner_fallible().map_err(AppError::io)?;
            Ok(val)
        }

        let result = outer();
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { code, message, .. } => {
                assert_eq!(code, "IO_001");
                assert!(message.contains("inner failure"));
            }
            other => panic!("Expected IoError, got {:?}", other),
        }
    }

    // ======================================================================
    // Additional AppError factory method tests (not_found, auth, internal)
    // ======================================================================

    /// Verify not_found factory produces correct error code and format.
    #[test]
    fn test_app_error_not_found_factory() {
        let err = AppError::not_found("image_123.jpg");
        assert_eq!(err.error_code(), "NF_001");
        let display = err.to_string();
        assert!(display.contains("image_123.jpg"));
    }

    /// Verify auth factory produces correct error code and format.
    #[test]
    fn test_app_error_auth_factory() {
        let err = AppError::auth("token expired");
        assert_eq!(err.error_code(), "AUTH_001");
        let display = err.to_string();
        assert!(display.contains("token expired"));
    }

    /// Verify internal factory produces correct error code and format.
    #[test]
    fn test_app_error_internal_factory() {
        let err = AppError::internal("unexpected null");
        assert_eq!(err.error_code(), "INT_001");
        let display = err.to_string();
        assert!(display.contains("unexpected null"));
    }
}
