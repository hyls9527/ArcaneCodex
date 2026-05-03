use crate::core::db::Database;
use crate::core::inference::InferenceProviderType;
use crate::utils::crypto;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceProviderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

#[tauri::command]
pub async fn get_inference_config(db: State<'_, Database>) -> AppResult<InferenceProviderConfig> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let provider = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_provider'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "lm_studio".to_string());

    let model = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_model'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Qwen2.5-VL-7B-Instruct".to_string());

    let api_key = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_api_key'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .filter(|k| !k.is_empty())
        .map(|k| crypto::decrypt_api_key(&k));

    let timeout_secs = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_timeout'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    Ok(InferenceProviderConfig {
        provider,
        model,
        api_key,
        timeout_secs,
    })
}

#[tauri::command]
pub async fn set_inference_provider(
    db: State<'_, Database>,
    provider: String,
    model: String,
    api_key: Option<String>,
) -> AppResult<()> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let ptype = match provider.as_str() {
        "lm_studio" => InferenceProviderType::LMStudio,
        "ollama" => InferenceProviderType::Ollama,
        "hermes" => InferenceProviderType::Hermes,
        "zhipu" => InferenceProviderType::Zhipu,
        "openai" => InferenceProviderType::OpenAI,
        "openrouter" => InferenceProviderType::OpenRouter,
        _ => {
            return Err(AppError::validation(format!(
                "不支持的推理提供者: {}",
                provider
            )))
        }
    };

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('inference_provider', ?1)",
        rusqlite::params![format!("{:?}", ptype).to_lowercase()],
    )
    .map_err(AppError::database)?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('inference_model', ?1)",
        rusqlite::params![model],
    )
    .map_err(AppError::database)?;

    let key_value = api_key.unwrap_or_default();
    let encrypted_key = crypto::encrypt_api_key(&key_value);
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('inference_api_key', ?1)",
        rusqlite::params![encrypted_key],
    )
    .map_err(AppError::database)?;

    info!("推理提供者已切换: {}", provider);
    Ok(())
}

#[tauri::command]
pub async fn test_inference_connection(db: State<'_, Database>) -> AppResult<String> {
    use crate::core::inference::{ProviderConfig, ProviderFactory};

    let conn = db.open_connection().map_err(AppError::database)?;

    let provider_type = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_provider'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "lm_studio".to_string());

    let model = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_model'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Qwen2.5-VL-7B-Instruct".to_string());

    let api_key = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'inference_api_key'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|k| crypto::decrypt_api_key(&k))
        .filter(|k| !k.is_empty());

    let ptype = match provider_type.as_str() {
        "zhipu" => InferenceProviderType::Zhipu,
        "openai" => InferenceProviderType::OpenAI,
        "openrouter" => InferenceProviderType::OpenRouter,
        _ => InferenceProviderType::LMStudio,
    };

    let config = ProviderConfig {
        provider_type: ptype,
        model,
        api_key,
        ..Default::default()
    };

    let provider = ProviderFactory::create(config)?;
    let models = provider.health_check().await?;

    Ok(format!("连接成功，可用模型: {}", models.join(", ")))
}

#[tauri::command]
pub async fn discover_available_models() -> AppResult<Vec<crate::core::inference::DiscoveredModel>>
{
    let models = crate::core::inference::ModelDiscoveryService::scan_all().await;
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Database;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_inference_settings.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.run_migrations().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_get_default_inference_config() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = 'inference_provider'",
            [],
            |row| row.get::<_, String>(0),
        );

        assert_eq!(result.unwrap(), "lm_studio");
    }

    #[test]
    fn test_provider_type_parsing() {
        assert!(matches!(
            match "lm_studio" {
                "lm_studio" => InferenceProviderType::LMStudio,
                "zhipu" => InferenceProviderType::Zhipu,
                _ => InferenceProviderType::LMStudio,
            },
            InferenceProviderType::LMStudio
        ));

        assert!(matches!(
            match "zhipu" {
                "lm_studio" => InferenceProviderType::LMStudio,
                "zhipu" => InferenceProviderType::Zhipu,
                _ => InferenceProviderType::LMStudio,
            },
            InferenceProviderType::Zhipu
        ));
    }
}
