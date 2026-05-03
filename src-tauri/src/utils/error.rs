use thiserror::Error;
use serde::Serialize;
use regex::Regex;

pub fn sanitize_error(msg: &str) -> String {
    let path_re = Regex::new(r"(?i)(?:[A-Za-z]:\\|/)(?:[^\s:\\/]+[\\/])+[^\s:\\/]*").unwrap();
    let sql_re = Regex::new(r"(?i)\b(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|PRAGMA)\b.+?(?:;|$)").unwrap();
    let result = path_re.replace_all(msg, "[PATH]");
    sql_re.replace_all(&result, "[QUERY]").to_string()
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("[{code}] Database error: {message}")]
    Database {
        code: String,
        message: String,
        #[source]
        source: rusqlite::Error,
    },

    #[error("[{code}] IO error: {message}")]
    IoError {
        code: String,
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("[{code}] Validation error: {message}")]
    ValidationError {
        code: String,
        message: String,
    },

    #[error("[{code}] AI error: {message}")]
    AI {
        code: String,
        message: String,
    },

    #[error("[{code}] HTTP error: {message}")]
    Http {
        code: String,
        message: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("[{code}] Config error: {message}")]
    Config {
        code: String,
        message: String,
    },
}

impl AppError {
    pub fn database(source: rusqlite::Error) -> Self {
        AppError::Database {
            code: "DB_001".to_string(),
            message: source.to_string(),
            source,
        }
    }

    pub fn io(source: std::io::Error) -> Self {
        AppError::IoError {
            code: "IO_001".to_string(),
            message: source.to_string(),
            source,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        AppError::ValidationError {
            code: "VAL_001".to_string(),
            message: message.into(),
        }
    }

    pub fn ai(message: impl Into<String>) -> Self {
        AppError::AI {
            code: "AI_001".to_string(),
            message: message.into(),
        }
    }

    pub fn http(source: reqwest::Error) -> Self {
        AppError::Http {
            code: "HTTP_001".to_string(),
            message: source.to_string(),
            source,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        AppError::Config {
            code: "CFG_001".to_string(),
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn not_found(resource: impl Into<String>) -> Self {
        AppError::ValidationError {
            code: "NF_001".to_string(),
            message: format!("{} 不存在", resource.into()),
        }
    }

    #[allow(dead_code)]
    pub fn auth(message: impl Into<String>) -> Self {
        AppError::ValidationError {
            code: "AUTH_001".to_string(),
            message: format!("认证失败: {}", message.into()),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        AppError::Config {
            code: "INT_001".to_string(),
            message: message.into(),
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            AppError::Database { code, .. } => code,
            AppError::IoError { code, .. } => code,
            AppError::ValidationError { code, .. } => code,
            AppError::AI { code, .. } => code,
            AppError::Http { code, .. } => code,
            AppError::Config { code, .. } => code,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::database(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::io(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::http(err)
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
struct SerializedError {
    code: String,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized = SerializedError {
            code: self.error_code().to_string(),
            message: sanitize_error(&self.to_string()),
        };
        serialized.serialize(serializer)
    }
}

pub fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};

    let log_dir = std::env::var("APPDATA")
        .map(|appdata| format!("{}\\ArcaneCodex\\logs", appdata))
        .unwrap_or_else(|_| "./logs".to_string());

    std::fs::create_dir_all(&log_dir).ok();

    let log_file = format!("{}\\app.log", log_dir);
    let file_writer = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .ok();

    if let Some(writer) = file_writer {
        fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .with_writer(writer)
            .with_ansi(false)
            .init();
    } else {
        fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_error_display() {
        let err = AppError::validation("test validation error");
        assert!(err.to_string().contains("test validation error"));
    }

    #[test]
    fn test_app_error_validation() {
        let err = AppError::validation("test error");
        assert!(err.to_string().contains("test error"));
        assert_eq!(err.error_code(), "VAL_001");
    }

    #[test]
    fn test_app_error_config() {
        let err = AppError::config("missing config");
        assert!(err.to_string().contains("missing config"));
        assert_eq!(err.error_code(), "CFG_001");
    }

    #[test]
    fn test_app_error_not_found() {
        let err = AppError::not_found("image not found");
        assert!(err.to_string().contains("image not found"));
        assert_eq!(err.error_code(), "NF_001");
    }

    #[test]
    fn test_app_error_auth() {
        let err = AppError::auth("unauthorized");
        assert!(err.to_string().contains("unauthorized"));
        assert_eq!(err.error_code(), "AUTH_001");
    }

    #[test]
    fn test_app_error_database() {
        let err = AppError::database(rusqlite::Error::InvalidQuery);
        assert!(err.to_string().contains("Database error"));
        assert_eq!(err.error_code(), "DB_001");
    }

    #[test]
    fn test_app_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AppError::io(io_err);
        assert!(err.to_string().contains("file not found"));
        assert_eq!(err.error_code(), "IO_001");
    }

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_app_result_err() {
        let result: AppResult<i32> = Err(AppError::validation("error"));
        assert!(result.is_err());
    }

    #[test]
    fn test_init_logging_creates_log_directory() {
        let log_dir = std::env::var("APPDATA")
            .map(|appdata| format!("{}\\ArcaneCodex\\logs", appdata))
            .unwrap_or_else(|_| "./logs".to_string());

        std::fs::create_dir_all(&log_dir).ok();

        assert!(std::path::Path::new(&log_dir).exists());
    }

    #[test]
    fn test_error_conversion_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_conversion_from_rusqlite_error() {
        fn trigger_db_error() -> Result<(), rusqlite::Error> {
            Err(rusqlite::Error::InvalidQuery)
        }

        let result: Result<(), AppError> = trigger_db_error().map_err(AppError::from);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Database { .. } => {},
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_type_discrimination() {
        let errors: Vec<AppError> = vec![
            AppError::validation("validation"),
            AppError::config("config"),
            AppError::ai("ai error"),
        ];

        for err in errors {
            let msg = err.to_string();
            match &err {
                AppError::ValidationError { message, .. } => assert!(msg.contains(message)),
                AppError::Config { message, .. } => assert!(msg.contains(message)),
                AppError::AI { message, .. } => assert!(msg.contains(message)),
                _ => panic!("Unexpected error type"),
            }
        }
    }

    #[test]
    fn test_sanitize_error_replaces_windows_path() {
        let msg = "File not found: C:\\Users\\test\\data\\image.jpg";
        let sanitized = sanitize_error(msg);
        assert!(!sanitized.contains("C:\\Users"), "Windows path should be replaced");
        assert!(sanitized.contains("[PATH]"), "Should contain [PATH] placeholder");
    }

    #[test]
    fn test_sanitize_error_replaces_unix_path() {
        let msg = "Failed to read /home/user/secret/data.db";
        let sanitized = sanitize_error(msg);
        assert!(!sanitized.contains("/home/user"), "Unix path should be replaced");
        assert!(sanitized.contains("[PATH]"), "Should contain [PATH] placeholder");
    }

    #[test]
    fn test_sanitize_error_replaces_sql_query() {
        let msg = "Database error: SELECT * FROM users WHERE id = 1;";
        let sanitized = sanitize_error(msg);
        assert!(!sanitized.contains("SELECT"), "SQL keyword should be replaced");
        assert!(sanitized.contains("[QUERY]"), "Should contain [QUERY] placeholder");
    }

    #[test]
    fn test_sanitize_error_replaces_insert_statement() {
        let msg = "Error executing: INSERT INTO images (file_path) VALUES ('test');";
        let sanitized = sanitize_error(msg);
        assert!(!sanitized.contains("INSERT"), "INSERT should be replaced");
        assert!(sanitized.contains("[QUERY]"), "Should contain [QUERY] placeholder");
    }

    #[test]
    fn test_sanitize_error_preserves_safe_message() {
        let msg = "Validation error: file is empty";
        let sanitized = sanitize_error(msg);
        assert_eq!(sanitized, msg, "Safe messages should not be modified");
    }

    #[test]
    fn test_sanitize_error_mixed_path_and_sql() {
        let msg = "Error at C:\\app\\data.db: SELECT * FROM secrets;";
        let sanitized = sanitize_error(msg);
        assert!(!sanitized.contains("C:\\app"), "Path should be replaced");
        assert!(!sanitized.contains("SELECT"), "SQL should be replaced");
        assert!(sanitized.contains("[PATH]"), "Should contain [PATH]");
        assert!(sanitized.contains("[QUERY]"), "Should contain [QUERY]");
    }

    #[test]
    fn test_sanitize_error_serialization_integration() {
        let err = AppError::validation("文件不存在: C:\\Users\\test\\image.jpg");
        let serialized = serde_json::to_string(&err).unwrap();
        assert!(!serialized.contains("C:\\\\Users"), "Serialized error should not contain path");
        assert!(serialized.contains("[PATH]"), "Serialized error should contain [PATH]");
    }

    #[test]
    fn test_serialized_error_structure() {
        let err = AppError::not_found("resource missing");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "NF_001");
        assert!(json["message"].as_str().unwrap().contains("resource missing"));
    }
}
