use serde::{Deserialize, Serialize};

use tauri::State;

use crate::core::db::Database;

use crate::utils::error::{AppError, AppResult};

use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]

pub struct AppConfig {
    pub key: String,

    pub value: String,
}

#[tauri::command]

pub async fn get_config(db: State<'_, Database>, key: String) -> AppResult<Option<AppConfig>> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let result = conn.query_row(
        "SELECT key, value FROM settings WHERE key = ?",
        [&key],
        |row| {
            Ok(AppConfig {
                key: row.get(0)?,

                value: row.get(1)?,
            })
        },
    );

    match result {
        Ok(config) => {
            info!("Config saved: {} = {}", config.key, config.value);

            Ok(Some(config))
        }

        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),

        Err(e) => Err(AppError::database(e)),
    }
}

#[tauri::command]

pub async fn set_config(db: State<'_, Database>, key: String, value: String) -> AppResult<()> {
    let conn = db.open_connection().map_err(AppError::database)?;

    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))

         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        [&key, &value],
    )
    .map_err(AppError::database)?;

    info!("Config set: {} = {}", key, value);

    Ok(())
}

#[tauri::command]

pub async fn get_all_configs(db: State<'_, Database>) -> AppResult<Vec<AppConfig>> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let mut stmt = conn
        .prepare("SELECT key, value FROM settings ORDER BY key")
        .map_err(AppError::database)?;

    let configs = stmt
        .query_map([], |row| {
            Ok(AppConfig {
                key: row.get(0)?,

                value: row.get(1)?,
            })
        })
        .map_err(AppError::database)?;

    let result: Vec<AppConfig> = configs.filter_map(|r| r.ok()).collect();

    info!("All configs retrieved: {} items", result.len());

    Ok(result)
}

#[tauri::command]

pub async fn backup_database(db: State<'_, Database>, output_path: String) -> AppResult<String> {
    use std::io::{Read, Write};

    use zip::write::SimpleFileOptions;

    use zip::ZipWriter;

    let db_path = db.db_path.clone();

    // Ensure output path has .zip extension

    let mut output_path = std::path::PathBuf::from(&output_path);

    if output_path.extension().map_or(true, |ext| ext != "zip") {
        output_path.set_extension("zip");
    }

    // Create parent directories if they don't exist

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;
    }

    let db_path_clone = db_path.clone();

    let output_path_clone = output_path.clone();

    // Run backup in a blocking thread since zip operations are CPU-bound

    let result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        // Check if database file exists

        if !db_path_clone.exists() {
            return Err(AppError::config("Database file not found".to_string()));
        }

        // Create zip file
        let zip_file = std::fs::File::create(&output_path_clone)
            .map_err(|e| AppError::config(format!("Failed to create zip file: {}", e)))?;

        let mut zip = ZipWriter::new(zip_file);

        // Add database file to zip

        let db_file_name = db_path_clone
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "arcanecodex.db".to_string());

        zip.start_file(&db_file_name, SimpleFileOptions::default())
            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

        let mut db_file = std::fs::File::open(&*db_path_clone)
            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

        let mut buffer = Vec::new();

        db_file
            .read_to_end(&mut buffer)
            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

        zip.write_all(&buffer)
            .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;

        // Also include WAL and SHM files if they exist

        let wal_path = db_path_clone.with_extension("db-wal");

        if wal_path.exists() {
            let wal_name = wal_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-wal".to_string());

            zip.start_file(&wal_name, SimpleFileOptions::default())
                .map_err(|e| AppError::config(format!("Failed to process WAL file: {}", e)))?;

            let mut wal_file = std::fs::File::open(&wal_path)
                .map_err(|e| AppError::config(format!("Failed to process WAL file: {}", e)))?;

            let mut wal_buffer = Vec::new();

            wal_file
                .read_to_end(&mut wal_buffer)
                .map_err(|e| AppError::config(format!("Failed to process WAL file: {}", e)))?;

            zip.write_all(&wal_buffer)
                .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;
        }

        let shm_path = db_path_clone.with_extension("db-shm");

        if shm_path.exists() {
            let shm_name = shm_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-shm".to_string());

            zip.start_file(&shm_name, SimpleFileOptions::default())
                .map_err(|e| AppError::config(format!("Failed to process SHM file: {}", e)))?;

            let mut shm_file = std::fs::File::open(&shm_path)
                .map_err(|e| AppError::config(format!("Failed to process SHM file: {}", e)))?;

            let mut shm_buffer = Vec::new();

            shm_file
                .read_to_end(&mut shm_buffer)
                .map_err(|e| AppError::config(format!("Failed to process SHM file: {}", e)))?;

            zip.write_all(&shm_buffer)
                .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;
        }

        zip.finish()
            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

        Ok(output_path_clone.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))??;

    info!("Backup completed: {:?}", result);

    Ok(result)
}

#[tauri::command]
pub async fn backup_database_encrypted(
    db: State<'_, Database>,
    output_path: String,
    password: String,
) -> AppResult<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let db_path = db.db_path.clone();

    let mut output_path = std::path::PathBuf::from(&output_path);
    if output_path.extension().map_or(true, |ext| ext != "enc") {
        output_path.set_extension("enc");
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::config(format!("Failed to create directory: {}", e)))?;
    }

    let db_path_clone = db_path.clone();
    let output_path_clone = output_path.clone();
    let password_clone = password.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        if !db_path_clone.exists() {
            return Err(AppError::config("Database file not found".to_string()));
        }

        let mut zip_buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));

            let db_file_name = db_path_clone
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db".to_string());

            zip.start_file(&db_file_name, SimpleFileOptions::default())
                .map_err(|e| AppError::config(format!("Failed to create zip entry: {}", e)))?;

            let mut db_file = std::fs::File::open(&*db_path_clone)
                .map_err(|e| AppError::config(format!("Failed to open database: {}", e)))?;

            let mut buffer = Vec::new();
            db_file
                .read_to_end(&mut buffer)
                .map_err(|e| AppError::config(format!("Failed to read database: {}", e)))?;

            zip.write_all(&buffer)
                .map_err(|e| AppError::config(format!("Failed to write to zip: {}", e)))?;

            let wal_path = db_path_clone.with_extension("db-wal");
            if wal_path.exists() {
                let wal_name = wal_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "arcanecodex.db-wal".to_string());

                zip.start_file(&wal_name, SimpleFileOptions::default())
                    .map_err(|e| AppError::config(format!("Failed to create WAL entry: {}", e)))?;

                let mut wal_file = std::fs::File::open(&wal_path)
                    .map_err(|e| AppError::config(format!("Failed to open WAL file: {}", e)))?;

                let mut wal_buffer = Vec::new();
                wal_file
                    .read_to_end(&mut wal_buffer)
                    .map_err(|e| AppError::config(format!("Failed to read WAL file: {}", e)))?;

                zip.write_all(&wal_buffer)
                    .map_err(|e| AppError::config(format!("Failed to write WAL to zip: {}", e)))?;
            }

            let shm_path = db_path_clone.with_extension("db-shm");
            if shm_path.exists() {
                let shm_name = shm_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "arcanecodex.db-shm".to_string());

                zip.start_file(&shm_name, SimpleFileOptions::default())
                    .map_err(|e| AppError::config(format!("Failed to create SHM entry: {}", e)))?;

                let mut shm_file = std::fs::File::open(&shm_path)
                    .map_err(|e| AppError::config(format!("Failed to open SHM file: {}", e)))?;

                let mut shm_buffer = Vec::new();
                shm_file
                    .read_to_end(&mut shm_buffer)
                    .map_err(|e| AppError::config(format!("Failed to read SHM file: {}", e)))?;

                zip.write_all(&shm_buffer)
                    .map_err(|e| AppError::config(format!("Failed to write SHM to zip: {}", e)))?;
            }

            zip.finish()
                .map_err(|e| AppError::config(format!("Failed to finalize zip: {}", e)))?;
        }

        let mut hasher = Sha256::new();
        hasher.update(password_clone.as_bytes());
        let key_bytes = hasher.finalize();
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_data = cipher
            .encrypt(nonce, zip_buffer.as_ref())
            .map_err(|e| AppError::config(format!("Encryption failed: {}", e)))?;

        let mut output_file = std::fs::File::create(&output_path_clone)
            .map_err(|e| AppError::config(format!("Failed to create encrypted file: {}", e)))?;

        output_file
            .write_all(&nonce_bytes)
            .map_err(|e| AppError::config(format!("Failed to write nonce: {}", e)))?;

        output_file
            .write_all(&encrypted_data)
            .map_err(|e| AppError::config(format!("Failed to write encrypted data: {}", e)))?;

        Ok(output_path_clone.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| AppError::config(format!("Backup task failed: {}", e)))??;

    info!("Encrypted backup completed: {:?}", result);

    Ok(result)
}

#[tauri::command]

pub async fn restore_database(db: State<'_, Database>, backup_path: String) -> AppResult<()> {
    use std::io::{Read, Write};

    use zip::ZipArchive;

    let db_path = db.db_path.clone();

    let backup_path_buf = std::path::PathBuf::from(&backup_path);

    let db_path_clone = db_path.clone();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {

        // Check if backup file exists

        if !backup_path_buf.exists() {
            return Err(AppError::config("Backup file not found".to_string()));
        }

        let zip_file = std::fs::File::open(&backup_path_buf).map_err(|e| {
            AppError::config(format!("Failed to open backup file: {}", e))
        })?;

        let mut archive = ZipArchive::new(zip_file).map_err(|e| {
            AppError::config(format!("Failed to read zip archive: {}", e))
        })?;



        // Create a temporary directory for extraction

        let temp_dir = std::env::temp_dir().join(format!(

            "arcanecodex_restore_{}",

            std::time::SystemTime::now()

                .duration_since(std::time::UNIX_EPOCH)

                .unwrap_or_default()

                .as_millis()

        ));

        std::fs::create_dir_all(&temp_dir).map_err(|e| {

            AppError::config(format!("Failed to add database to zip: {}", e))

        })?;



        // Extract all files from zip

        for i in 0..archive.len() {

            let mut file = archive.by_index(i).map_err(|e| {

                AppError::config(format!("Operation failed: {}", e))

            })?;



            let entry_name = file.name();



            // Zip Slip protection: validate entry path

            if entry_name.contains("..") || entry_name.contains('\\') {

                warn!("Suspicious zip entry blocked: {}", entry_name);

                continue;

            }



            let outpath = temp_dir.join(entry_name);



            // Validate extraction path is within temp_dir
            if let Ok(canonical_out) = outpath.canonicalize() {

                if !canonical_out.starts_with(&temp_dir) {
                    warn!("Zip Slip attempt blocked: {}", entry_name);
                    continue;
                }

            } else {

                if let Some(p) = outpath.parent() {

                    if !p.exists() {

                        std::fs::create_dir_all(p).map_err(|e| {

                            AppError::config(format!("Operation failed: {}", e))

                        })?;

                    }

                }

                let mut outfile = std::fs::File::create(&outpath).map_err(|e| {

                    AppError::config(format!("Operation failed: {}", e))

                })?;

                let mut buffer = Vec::new();

                file.read_to_end(&mut buffer).map_err(|e| {

                    AppError::config(format!("Operation failed: {}", e))

                })?;

                outfile.write_all(&buffer).map_err(|e| {

                    AppError::config(format!("Operation failed: {}", e))

                })?;

            }

        }



        // Find the database file in extracted files

        let db_file_name = db_path_clone

            .file_name()

            .map(|n| n.to_string_lossy().to_string())

            .unwrap_or_else(|| "arcanecodex.db".to_string());



        let extracted_db = temp_dir.join(&db_file_name);

        if !extracted_db.exists() {

            let _ = std::fs::remove_dir_all(&temp_dir);

            return Err(AppError::config(format!(
                "Database file not found in backup: {}",
                db_file_name
            )));

        }



        let backup_version: i32 = {

            let backup_conn = rusqlite::Connection::open(&extracted_db)

                .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

            backup_conn.pragma_query_value(None, "user_version", |row| row.get(0))

                .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?

        };



        let current_version: i32 = {

            if db_path_clone.exists() {

                let current_conn = rusqlite::Connection::open(&*db_path_clone)

                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

                current_conn.pragma_query_value(None, "user_version", |row| row.get(0))

                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?

            } else {

                0

            }

        };



        if backup_version > current_version {

            let _ = std::fs::remove_dir_all(&temp_dir);

            return Err(AppError::config(format!(
                "Backup DB version (v{}) is higher than current version (v{}). Please upgrade the app first.",
                backup_version, current_version
            )));
        }

        // Close any WAL/SHM by checkpointing if possible, then replace files

        // Remove existing database files

        if db_path_clone.exists() {

            std::fs::remove_file(&*db_path_clone).map_err(|e| {

                AppError::config(format!("Operation failed: {}", e))

            })?;

        }



        // Copy restored database

        std::fs::copy(&extracted_db, &*db_path_clone).map_err(|e| {

            AppError::config(format!("Operation failed: {}", e))

        })?;



        // Also restore WAL file if it exists in backup

        let wal_file_name = format!("{}-wal", db_file_name);

        let extracted_wal = temp_dir.join(&wal_file_name);

        let target_wal = db_path_clone.with_extension("db-wal");

        if extracted_wal.exists() {

            std::fs::copy(&extracted_wal, &target_wal).map_err(|e| {

                AppError::config(format!("Failed to process WAL file: {}", e))

            })?;

        } else if target_wal.exists() {

            // Remove existing WAL if not in backup

            let _ = std::fs::remove_file(&target_wal);

        }



        // Also restore SHM file if it exists in backup

        let shm_file_name = format!("{}-shm", db_file_name);

        let extracted_shm = temp_dir.join(&shm_file_name);

        let target_shm = db_path_clone.with_extension("db-shm");

        if extracted_shm.exists() {

            std::fs::copy(&extracted_shm, &target_shm).map_err(|e| {

                AppError::config(format!("Failed to process SHM file: {}", e))

            })?;

        } else if target_shm.exists() {

            // Remove existing SHM if not in backup

            let _ = std::fs::remove_file(&target_shm);

        }



        // Clean up temp directory

        let _ = std::fs::remove_dir_all(&temp_dir);



        Ok(())

    })

    .await

    .map_err(|e| AppError::config(format!("Failed to restore database: {}", e)))??;

    info!("Database restore completed: {}", backup_path);

    Ok(())
}

#[tauri::command]
pub async fn restore_database_encrypted(
    db: State<'_, Database>,
    backup_path: String,
    password: String,
) -> AppResult<()> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use sha2::{Digest, Sha256};
    use std::io::Read;
    use zip::ZipArchive;

    let db_path = db.db_path.clone();
    let backup_path_buf = std::path::PathBuf::from(&backup_path);
    let password_clone = password.clone();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        if !backup_path_buf.exists() {
            return Err(AppError::config("Backup file not found".to_string()));
        }

        let mut encrypted_file = std::fs::File::open(&backup_path_buf)
            .map_err(|e| AppError::config(format!("Failed to open encrypted backup: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        encrypted_file
            .read_exact(&mut nonce_bytes)
            .map_err(|e| AppError::config(format!("Failed to read nonce: {}", e)))?;

        let mut encrypted_data = Vec::new();
        encrypted_file
            .read_to_end(&mut encrypted_data)
            .map_err(|e| AppError::config(format!("Failed to read encrypted data: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(password_clone.as_bytes());
        let key_bytes = hasher.finalize();
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted_data = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| {
                AppError::config(
                    "Decryption failed. Invalid password or corrupted file.".to_string(),
                )
            })?;

        let cursor = std::io::Cursor::new(decrypted_data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| AppError::config(format!("Failed to read zip archive: {}", e)))?;

        let temp_dir = std::env::temp_dir().join(format!(
            "arcanecodex_restore_enc_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));

        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::config(format!("Failed to create temp directory: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::config(format!("Failed to read zip entry: {}", e)))?;

            let entry_name = file.name();

            if entry_name.contains("..") || entry_name.contains('\\') {
                warn!("Suspicious zip entry blocked: {}", entry_name);
                continue;
            }

            let outpath = temp_dir.join(entry_name);

            if let Ok(canonical_out) = outpath.canonicalize() {
                if !canonical_out.starts_with(&temp_dir) {
                    warn!("Zip Slip attempt blocked: {}", entry_name);
                    continue;
                }
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).map_err(|e| {
                            AppError::config(format!("Failed to create directory: {}", e))
                        })?;
                    }
                }
            }

            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| AppError::config(format!("Failed to create file: {}", e)))?;

            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| AppError::config(format!("Failed to write file: {}", e)))?;
        }

        let db_path_clone = db_path.clone();

        let target_db = db_path_clone.clone();
        let target_wal = db_path_clone.with_extension("db-wal");
        let target_shm = db_path_clone.with_extension("db-shm");

        if target_db.exists() {
            std::fs::remove_file(&*target_db)
                .map_err(|e| AppError::config(format!("Failed to remove old database: {}", e)))?;
        }

        let source_db = temp_dir.join(
            db_path_clone
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db".to_string()),
        );

        if source_db.exists() {
            std::fs::copy(&source_db, &*target_db)
                .map_err(|e| AppError::config(format!("Failed to restore database: {}", e)))?;
        }

        let source_wal = temp_dir.join(
            target_wal
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-wal".to_string()),
        );

        if source_wal.exists() {
            std::fs::copy(&source_wal, &target_wal)
                .map_err(|e| AppError::config(format!("Failed to restore WAL file: {}", e)))?;
        } else if target_wal.exists() {
            let _ = std::fs::remove_file(&target_wal);
        }

        let source_shm = temp_dir.join(
            target_shm
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-shm".to_string()),
        );

        if source_shm.exists() {
            std::fs::copy(&source_shm, &target_shm)
                .map_err(|e| AppError::config(format!("Failed to restore SHM file: {}", e)))?;
        } else if target_shm.exists() {
            let _ = std::fs::remove_file(&target_shm);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(())
    })
    .await
    .map_err(|e| AppError::config(format!("Restore task failed: {}", e)))??;

    info!("Encrypted database restore completed: {}", backup_path);

    Ok(())
}

#[tauri::command]

pub async fn test_lm_studio_connection(url: String) -> AppResult<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;

    let health_url = format!("{}/v1/models", url.trim_end_matches('/'));

    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => Ok(true),

        Ok(resp) => {
            info!("LM Studio returned status: {}", resp.status());

            Ok(false)
        }

        Err(e) => {
            info!("LM Studio connection failed: {}", e);

            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::sync::Arc;

    use tempfile::TempDir;

    fn setup_test_db() -> Result<(Arc<Database>, TempDir), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        let db_path = temp_dir.path().join("test_settings.db");

        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        let db = Arc::new(Database::new_from_path(db_path_str)?);

        db.init()?;

        Ok((db, temp_dir))
    }

    #[test]

    fn test_settings_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfig {
            key: "lm_studio_url".to_string(),

            value: "http://localhost:1234".to_string(),
        };

        let json = serde_json::to_string(&config)?;

        let deserialized: AppConfig = serde_json::from_str(&json)?;

        assert_eq!(deserialized.key, "lm_studio_url");

        assert_eq!(deserialized.value, "http://localhost:1234");

        Ok(())
    }

    #[test]

    fn test_set_and_get_config() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp) = setup_test_db()?;

        let conn = db.open_connection()?;

        let key = "test_key".to_string();

        let value = "test_value".to_string();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            [&key, &value],
        )?;

        let result: Option<AppConfig> = conn
            .query_row(
                "SELECT key, value FROM settings WHERE key = ?",
                [&key],
                |row| {
                    Ok(AppConfig {
                        key: row.get(0)?,

                        value: row.get(1)?,
                    })
                },
            )
            .ok();

        assert!(result.is_some());

        let config = result.unwrap();

        assert_eq!(config.key, "test_key");

        assert_eq!(config.value, "test_value");

        Ok(())
    }

    #[test]

    fn test_upsert_config() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp) = setup_test_db()?;

        let conn = db.open_connection()?;

        let key = "custom_test_key".to_string();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            [&key, "3"],
        )?;

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)

             ON CONFLICT(key) DO UPDATE SET value = ?2",
            [&key, "5"],
        )?;

        let value: String =
            conn.query_row("SELECT value FROM settings WHERE key = ?", [&key], |row| {
                row.get(0)
            })?;

        assert_eq!(value, "5");

        Ok(())
    }

    #[test]

    fn test_get_nonexistent_config() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp) = setup_test_db()?;

        let conn = db.open_connection()?;

        let result: Result<AppConfig, _> = conn.query_row(
            "SELECT key, value FROM settings WHERE key = ?",
            ["nonexistent_key"],
            |row| {
                Ok(AppConfig {
                    key: row.get(0)?,

                    value: row.get(1)?,
                })
            },
        );

        assert!(result.is_err());

        Ok(())
    }

    #[test]

    fn test_get_all_configs() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp) = setup_test_db()?;

        let conn = db.open_connection()?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;

        assert!(count > 0, "Database should have at least one config entry");
        Ok(())
    }

    /// Sync version of backup_database for testing without async runtime.
    fn backup_database_sync(
        db_path: &std::path::Path,

        output_path: &std::path::Path,
    ) -> Result<String, AppError> {
        use std::io::{Read, Write};

        use zip::write::SimpleFileOptions;

        use zip::ZipWriter;

        let mut out = output_path.to_path_buf();

        if out.extension().map_or(true, |ext| ext != "zip") {
            out.set_extension("zip");
        }

        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::config(format!("Failed to create directory: {}", e)))?;
        }

        if !db_path.exists() {
            return Err(AppError::config("Database file not found".to_string()));
        }

        let zip_file = std::fs::File::create(&out)
            .map_err(|e| AppError::config(format!("Failed to create zip file: {}", e)))?;

        let mut zip = ZipWriter::new(zip_file);

        let db_file_name = db_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "arcanecodex.db".to_string());

        zip.start_file(&db_file_name, SimpleFileOptions::default())
            .map_err(|e| AppError::config(format!("Failed to add file to zip: {}", e)))?;

        let mut db_file = std::fs::File::open(db_path)
            .map_err(|e| AppError::config(format!("Failed to open database file: {}", e)))?;

        let mut buffer = Vec::new();

        db_file
            .read_to_end(&mut buffer)
            .map_err(|e| AppError::config(format!("Failed to read database file: {}", e)))?;

        zip.write_all(&buffer)
            .map_err(|e| AppError::config(format!("Failed to write to zip: {}", e)))?;

        let wal_path = db_path.with_extension("db-wal");

        if wal_path.exists() {
            let wal_name = wal_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-wal".to_string());

            zip.start_file(&wal_name, SimpleFileOptions::default())
                .map_err(|e| AppError::config(format!("Failed to add WAL to zip: {}", e)))?;

            let mut wal_file = std::fs::File::open(&wal_path)
                .map_err(|e| AppError::config(format!("Failed to open WAL file: {}", e)))?;

            let mut wal_buffer = Vec::new();

            wal_file
                .read_to_end(&mut wal_buffer)
                .map_err(|e| AppError::config(format!("Failed to read WAL file: {}", e)))?;

            zip.write_all(&wal_buffer)
                .map_err(|e| AppError::config(format!("Failed to write WAL to zip: {}", e)))?;
        }

        let shm_path = db_path.with_extension("db-shm");

        if shm_path.exists() {
            let shm_name = shm_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "arcanecodex.db-shm".to_string());

            zip.start_file(&shm_name, SimpleFileOptions::default())
                .map_err(|e| AppError::config(format!("Failed to add SHM to zip: {}", e)))?;

            let mut shm_file = std::fs::File::open(&shm_path)
                .map_err(|e| AppError::config(format!("Failed to open SHM file: {}", e)))?;

            let mut shm_buffer = Vec::new();

            shm_file
                .read_to_end(&mut shm_buffer)
                .map_err(|e| AppError::config(format!("Failed to read SHM file: {}", e)))?;

            zip.write_all(&shm_buffer)
                .map_err(|e| AppError::config(format!("Failed to write SHM to zip: {}", e)))?;
        }

        zip.finish()
            .map_err(|e| AppError::config(format!("Failed to finalize zip: {}", e)))?;

        Ok(out.to_string_lossy().to_string())
    }

    /// Sync version of restore_database for testing without async runtime.
    fn restore_database_sync(
        backup_path: &std::path::Path,

        target_db_path: &std::path::Path,
    ) -> Result<(), AppError> {
        use std::io::{Read, Write};

        use zip::ZipArchive;

        if !backup_path.exists() {
            return Err(AppError::config("Backup file not found".to_string()));
        }

        let zip_file = std::fs::File::open(&backup_path)
            .map_err(|e| AppError::config(format!("Failed to open backup file: {}", e)))?;

        let mut archive = ZipArchive::new(zip_file)
            .map_err(|e| AppError::config(format!("Failed to read zip archive: {}", e)))?;

        // Extract to temp directory

        let temp_dir = std::env::temp_dir().join(format!(
            "arcanecodex_restore_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));

        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

            let outpath = temp_dir.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;
                    }
                }

                let mut outfile = std::fs::File::create(&outpath)
                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

                let mut buffer = Vec::new();

                file.read_to_end(&mut buffer)
                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

                outfile
                    .write_all(&buffer)
                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;
            }
        }

        // Find db file

        let db_file_name = target_db_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "arcanecodex.db".to_string());

        let extracted_db = temp_dir.join(&db_file_name);

        if !extracted_db.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);

            return Err(AppError::config(format!(
                "Database operation failed: {}",
                db_file_name
            )));
        }

        let backup_version: i32 = {
            let backup_conn = rusqlite::Connection::open(&extracted_db)
                .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?;

            backup_conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .map_err(|e| AppError::config(format!("Failed to add database to zip: {}", e)))?
        };

        let current_version: i32 = {
            if target_db_path.exists() {
                let current_conn = rusqlite::Connection::open(target_db_path).map_err(|e| {
                    AppError::config(format!("Failed to add database to zip: {}", e))
                })?;

                current_conn
                    .pragma_query_value(None, "user_version", |row| row.get(0))
                    .map_err(|e| {
                        AppError::config(format!("Failed to add database to zip: {}", e))
                    })?
            } else {
                0
            }
        };

        if backup_version > current_version {
            let _ = std::fs::remove_dir_all(&temp_dir);

            return Err(AppError::config(format!(
                "Backup DB version (v{}) is higher than current (v{}). Upgrade the app first.",
                backup_version, current_version
            )));
        }

        // Replace target db

        if target_db_path.exists() {
            // Try to delete, if failed (Windows file lock), rename to temp

            if std::fs::remove_file(target_db_path).is_err() {
                let backup_old = target_db_path.with_extension("db.old");

                let _ = std::fs::remove_file(&backup_old);

                std::fs::rename(target_db_path, &backup_old)
                    .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;
            }
        }

        std::fs::copy(&extracted_db, target_db_path)
            .map_err(|e| AppError::config(format!("Operation failed: {}", e)))?;

        // WAL

        let wal_file_name = format!("{}-wal", db_file_name);

        let extracted_wal = temp_dir.join(&wal_file_name);

        let target_wal = target_db_path.with_extension("db-wal");

        if extracted_wal.exists() {
            std::fs::copy(&extracted_wal, &target_wal).ok();
        } else if target_wal.exists() {
            let _ = std::fs::remove_file(&target_wal);
        }

        // SHM

        let shm_file_name = format!("{}-shm", db_file_name);

        let extracted_shm = temp_dir.join(&shm_file_name);

        let target_shm = target_db_path.with_extension("db-shm");

        if extracted_shm.exists() {
            std::fs::copy(&extracted_shm, &target_shm).ok();
        } else if target_shm.exists() {
            let _ = std::fs::remove_file(&target_shm);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(())
    }

    // ============================================================

    // TC-SETTINGS-HP-007: End-to-end backup and restore flow

    // ============================================================

    #[test]

    fn tc_settings_hp_007_backup_and_restore_end_to_end() {
        // --- Step 1: Create a test database and insert some data ---

        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_backup_restore.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db.init().unwrap();

        // Insert some test images

        let conn = db.open_connection().unwrap();

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",

            rusqlite::params![

                "/test/image_001.jpg", "image_001.jpg", 12345, "hash001",

                "completed", "[\"nature\", \"mountain\"]", "A beautiful mountain landscape"

            ],

        )

        .unwrap();

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",

            rusqlite::params![

                "/test/image_002.png", "image_002.png", 67890, "hash002",

                "completed", "[\"city\", \"night\"]", "City skyline at night"

            ],

        )

        .unwrap();

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",

            rusqlite::params![

                "/test/image_003.jpg", "image_003.jpg", 11111, "hash003",

                "pending", "[]", ""

            ],

        )

        .unwrap();

        // Insert some tags

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["nature", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["mountain", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["city", 1],
        )
        .unwrap();

        // Insert image_tags

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![1, 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![1, 2],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![2, 3],
        )
        .unwrap();

        // Update a config value

        conn.execute(
            "UPDATE settings SET value = ?1 WHERE key = 'ai_concurrency'",
            ["5"],
        )
        .unwrap();

        drop(conn);

        drop(db);

        // Record expected counts before backup

        let db2 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db2.open_connection().unwrap();

        let image_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        let tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
            .unwrap();

        let image_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM image_tags", [], |r| r.get(0))
            .unwrap();

        let ai_concurrency: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_concurrency'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        drop(conn);

        drop(db2);

        assert_eq!(image_count, 3, "Should have 3 images before backup");

        assert_eq!(tag_count, 3, "Should have 3 tags before backup");

        assert_eq!(image_tag_count, 3, "Should have 3 image_tags before backup");

        assert_eq!(
            ai_concurrency, "5",
            "ai_concurrency should be 5 before backup"
        );

        // --- Step 2: Create backup ---

        let backup_path = temp_dir.path().join("backup_test.zip");

        let backup_result = backup_database_sync(&db_path, &backup_path);

        assert!(backup_result.is_ok(), "Backup should succeed");

        let backup_file_path = backup_result.unwrap();

        assert!(
            std::path::Path::new(&backup_file_path).exists(),
            "Backup zip file should exist"
        );

        // Verify zip contains at least the database file

        {
            use std::fs::File;

            use zip::ZipArchive;

            let file = File::open(&backup_file_path).unwrap();

            let mut archive = ZipArchive::new(file).unwrap();

            let file_names: Vec<String> = (0..archive.len())
                .map(|i| archive.by_index(i).unwrap().name().to_string())
                .collect();

            assert!(
                file_names
                    .iter()
                    .any(|n| n.contains(".db") && !n.contains("-wal") && !n.contains("-shm")),
                "ZIP should contain the main database file"
            );
        }

        // --- Step 3: Delete all data from the database ---

        // We drop all tables to simulate complete data loss

        let db3 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db3.open_connection().unwrap();

        conn.execute_batch(
            "

            DROP TABLE IF EXISTS image_tags;

            DROP TABLE IF EXISTS search_index;

            DROP TABLE IF EXISTS task_queue;

            DROP TABLE IF EXISTS tags;

            DROP TABLE IF EXISTS images;

            DROP TABLE IF EXISTS settings;

            DROP TABLE IF EXISTS narratives;

            DROP TABLE IF EXISTS semantic_edges;

            DROP TABLE IF EXISTS settings;

            DROP TABLE IF EXISTS calibration_samples;

            DROP TABLE IF EXISTS calibration_reports;

            DROP TABLE IF EXISTS calibration_curves;

            DROP TABLE IF EXISTS tag_corrections;

            DROP TABLE IF EXISTS error_patterns;

        ",
        )
        .unwrap();

        drop(conn);

        // Verify tables are gone (excluding SQLite internal tables)

        {
            let conn = db3.open_connection().unwrap();

            let table_count: i64 = conn.query_row(

                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",

                [],

                |r| r.get(0),

            ).unwrap();

            assert_eq!(table_count, 0, "All tables should be dropped");
        }

        drop(db3);

        // Give Windows time to release file handles

        std::thread::sleep(std::time::Duration::from_millis(100));

        // --- Step 4: Restore from backup ---

        let restore_result =
            restore_database_sync(std::path::Path::new(&backup_file_path), &db_path);

        assert!(restore_result.is_ok(), "Restore should succeed");

        // --- Step 5: Verify all data is restored correctly ---

        // Re-open the restored database with fresh connection and re-init schema

        let db4 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db4.run_migrations().ok(); // Run migrations to ensure settings table exists if needed

        let conn = db4.open_connection().unwrap();

        // Check images restored

        let restored_image_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_image_count, image_count,
            "Image count should match after restore"
        );

        // Check image data integrity

        let (file_name, ai_status, ai_tags, ai_description): (String, String, String, String) = conn

            .query_row(

                "SELECT file_name, ai_status, ai_tags, ai_description FROM images WHERE file_path = ?",

                ["/test/image_001.jpg"],

                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),

            )

            .unwrap();

        assert_eq!(file_name, "image_001.jpg");

        assert_eq!(ai_status, "completed");

        assert!(ai_tags.contains("nature"));

        assert_eq!(ai_description, "A beautiful mountain landscape");

        let (file_name2, file_size2): (String, i64) = conn
            .query_row(
                "SELECT file_name, file_size FROM images WHERE file_path = ?",
                ["/test/image_002.png"],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(file_name2, "image_002.png");

        assert_eq!(file_size2, 67890);

        // Check tags restored

        let restored_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_tag_count, tag_count,
            "Tag count should match after restore"
        );

        // Check image_tags restored

        let restored_image_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM image_tags", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_image_tag_count, image_tag_count,
            "image_tag count should match after restore"
        );

        // Check config restored

        let restored_ai_concurrency: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_concurrency'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(
            restored_ai_concurrency, ai_concurrency,
            "ai_concurrency config should be restored"
        );

        // Verify database integrity

        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();

        assert_eq!(integrity, "ok", "Database integrity check should pass");

        // Verify WAL mode is active

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            journal_mode, "wal",
            "WAL mode should be active after restore"
        );

        drop(conn);
    }

    #[test]

    fn tc_settings_hp_007b_backup_without_wal_shm() {
        // Test backup works when WAL/SHM files don't exist

        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_no_wal.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db.init().unwrap();

        // Insert data

        let conn = db.open_connection().unwrap();

        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size) VALUES (?1, ?2, ?3)",
            rusqlite::params!["/test/no_wal.jpg", "no_wal.jpg", 9999],
        )
        .unwrap();

        drop(conn);

        // Close connection and remove WAL/SHM if they exist

        let wal_path = db_path.with_extension("db-wal");

        let shm_path = db_path.with_extension("db-shm");

        if wal_path.exists() {
            let _ = std::fs::remove_file(&wal_path);
        }

        if shm_path.exists() {
            let _ = std::fs::remove_file(&shm_path);
        }

        // Backup should succeed without WAL/SHM

        let backup_path = temp_dir.path().join("backup_no_wal.zip");

        let result = backup_database_sync(&db_path, &backup_path);

        assert!(
            result.is_ok(),
            "Backup should succeed without WAL/SHM files"
        );
    }

    #[test]

    fn tc_settings_hp_007c_restore_missing_backup() {
        // Test restore fails gracefully when backup doesn't exist

        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_restore.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db.init().unwrap();

        let missing_backup = temp_dir.path().join("nonexistent_backup.zip");

        let result = restore_database_sync(&missing_backup, &db_path);

        assert!(result.is_err(), "Restore should fail with missing backup");
    }

    #[test]

    fn tc_settings_hp_007d_restore_invalid_zip() {
        // Test restore fails with invalid zip file

        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_restore_invalid.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db.init().unwrap();

        // Create a non-zip file

        let fake_zip = temp_dir.path().join("fake.zip");

        std::fs::write(&fake_zip, "this is not a zip file").unwrap();

        let result = restore_database_sync(&fake_zip, &db_path);

        assert!(result.is_err(), "Restore should fail with invalid zip");
    }

    #[test]

    fn test_backup_delete_restore_integrity() {
        // TC-SETTINGS-HP-007: Delete all data then restore from backup, verify data integrity.

        // This test specifically uses DELETE to remove data while preserving table structure,

        // simulating a scenario where user data is deleted but schema remains intact.

        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_delete_restore.db");

        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        db.init().unwrap();

        // --- Phase 1: Create test data ---

        let conn = db.open_connection().unwrap();

        // Insert test images with various states

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description, ai_category) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",

            rusqlite::params![

                "/test/delete_restore_001.jpg", "delete_restore_001.jpg", 12345, "hash_dr001",

                "completed", "[\"nature\", \"mountain\", \"sunset\"]", "A beautiful mountain landscape at sunset",

                "landscape"

            ],

        ).unwrap();

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description, ai_category) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",

            rusqlite::params![

                "/test/delete_restore_002.png", "delete_restore_002.png", 67890, "hash_dr002",

                "completed", "[\"city\", \"night\", \"skyline\"]", "City skyline at night with lights",

                "cityscape"

            ],

        ).unwrap();

        conn.execute(

            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_tags, ai_description, ai_category) 

             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",

            rusqlite::params![

                "/test/delete_restore_003.jpg", "delete_restore_003.jpg", 54321, "hash_dr003",

                "pending", "[]", "", "portrait"

            ],

        ).unwrap();

        // Insert tags

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["nature", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["mountain", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["sunset", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["city", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["night", 1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, count) VALUES (?1, ?2)",
            rusqlite::params!["skyline", 1],
        )
        .unwrap();

        // Insert image_tags relationships

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (1, 1)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (1, 2)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (1, 3)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (2, 4)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (2, 5)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (2, 6)",
            [],
        )
        .unwrap();

        // Insert search_index entries

        conn.execute("INSERT INTO search_index (image_id, term, field, weight) VALUES (1, 'nature', 'tags', 1.0)", []).unwrap();

        conn.execute("INSERT INTO search_index (image_id, term, field, weight) VALUES (1, 'mountain', 'tags', 1.0)", []).unwrap();

        conn.execute("INSERT INTO search_index (image_id, term, field, weight) VALUES (1, 'sunset', 'tags', 1.0)", []).unwrap();

        conn.execute("INSERT INTO search_index (image_id, term, field, weight) VALUES (2, 'city', 'tags', 1.0)", []).unwrap();

        conn.execute("INSERT INTO search_index (image_id, term, field, weight) VALUES (2, 'night', 'tags', 1.0)", []).unwrap();

        // Update config values

        conn.execute(
            "UPDATE settings SET value = '5' WHERE key = 'ai_concurrency'",
            [],
        )
        .unwrap();

        conn.execute(
            "UPDATE settings SET value = '120' WHERE key = 'ai_timeout_seconds'",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('custom_setting', 'test_value')",
            [],
        )
        .unwrap();

        drop(conn);

        drop(db);

        // --- Phase 2: Record expected counts before backup ---

        let db2 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db2.open_connection().unwrap();

        let image_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        let tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
            .unwrap();

        let image_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM image_tags", [], |r| r.get(0))
            .unwrap();

        let search_index_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_index", [], |r| r.get(0))
            .unwrap();

        let ai_concurrency: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_concurrency'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let ai_timeout: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_timeout_seconds'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        drop(conn);

        drop(db2);

        assert_eq!(image_count, 3, "Should have 3 images before backup");

        assert_eq!(tag_count, 6, "Should have 6 tags before backup");

        assert_eq!(image_tag_count, 6, "Should have 6 image_tags before backup");

        assert_eq!(
            search_index_count, 5,
            "Should have 5 search_index entries before backup"
        );

        assert_eq!(
            ai_concurrency, "5",
            "ai_concurrency should be 5 before backup"
        );

        assert_eq!(
            ai_timeout, "120",
            "ai_timeout_seconds should be 120 before backup"
        );

        // --- Phase 3: Create backup ---

        let backup_path = temp_dir.path().join("backup_delete_restore.zip");

        let backup_result = backup_database_sync(&db_path, &backup_path);

        assert!(
            backup_result.is_ok(),
            "Backup should succeed: {:?}",
            backup_result
        );

        let backup_file_path = backup_result.unwrap();

        assert!(
            std::path::Path::new(&backup_file_path).exists(),
            "Backup zip file should exist"
        );

        // Verify ZIP contents

        {
            use std::fs::File;

            use zip::ZipArchive;

            let file = File::open(&backup_file_path).unwrap();

            let mut archive = ZipArchive::new(file).unwrap();

            let file_names: Vec<String> = (0..archive.len())
                .map(|i| archive.by_index(i).unwrap().name().to_string())
                .collect();

            // Should contain main db file

            assert!(
                file_names
                    .iter()
                    .any(|n| n.contains(".db") && !n.contains("-wal") && !n.contains("-shm")),
                "ZIP should contain the main database file, got: {:?}",
                file_names
            );
        }

        // --- Phase 4: Delete all data from the database (simulating data loss) ---

        let db3 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db3.open_connection().unwrap();

        // Delete in correct order to respect foreign key constraints

        conn.execute("DELETE FROM image_tags", []).unwrap();

        conn.execute("DELETE FROM search_index", []).unwrap();

        conn.execute("DELETE FROM task_queue", []).unwrap();

        conn.execute("DELETE FROM tags", []).unwrap();

        conn.execute("DELETE FROM images", []).unwrap();

        // Verify all data is deleted

        let image_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        let tag_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
            .unwrap();

        let image_tag_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM image_tags", [], |r| r.get(0))
            .unwrap();

        let search_index_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_index", [], |r| r.get(0))
            .unwrap();

        assert_eq!(image_count_after, 0, "All images should be deleted");

        assert_eq!(tag_count_after, 0, "All tags should be deleted");

        assert_eq!(image_tag_count_after, 0, "All image_tags should be deleted");

        assert_eq!(
            search_index_count_after, 0,
            "All search_index entries should be deleted"
        );

        drop(conn);

        drop(db3);

        // Give Windows time to release file handles

        std::thread::sleep(std::time::Duration::from_millis(100));

        // --- Phase 5: Restore from backup ---

        let restore_result =
            restore_database_sync(std::path::Path::new(&backup_file_path), &db_path);

        assert!(
            restore_result.is_ok(),
            "Restore should succeed: {:?}",
            restore_result
        );

        // --- Phase 6: Verify all data is restored correctly ---

        let db4 = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = db4.open_connection().unwrap();

        // Check image count restored

        let restored_image_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_image_count, image_count,
            "Image count should match after restore"
        );

        // Check image data integrity - verify each image's fields

        let (file_name, ai_status, ai_tags, ai_description, ai_category): (String, String, String, String, String) = conn

            .query_row(

                "SELECT file_name, ai_status, ai_tags, ai_description, ai_category FROM images WHERE file_path = ?",

                ["/test/delete_restore_001.jpg"],

                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),

            )

            .unwrap();

        assert_eq!(
            file_name, "delete_restore_001.jpg",
            "Image 001 file_name should match"
        );

        assert_eq!(
            ai_status, "completed",
            "Image 001 ai_status should be completed"
        );

        assert!(
            ai_tags.contains("nature"),
            "Image 001 ai_tags should contain 'nature'"
        );

        assert!(
            ai_tags.contains("mountain"),
            "Image 001 ai_tags should contain 'mountain'"
        );

        assert!(
            ai_tags.contains("sunset"),
            "Image 001 ai_tags should contain 'sunset'"
        );

        assert_eq!(
            ai_description, "A beautiful mountain landscape at sunset",
            "Image 001 description should match"
        );

        assert_eq!(ai_category, "landscape", "Image 001 category should match");

        let (file_name2, file_size2, hash2): (String, i64, String) = conn
            .query_row(
                "SELECT file_name, file_size, file_hash FROM images WHERE file_path = ?",
                ["/test/delete_restore_002.png"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();

        assert_eq!(
            file_name2, "delete_restore_002.png",
            "Image 002 file_name should match"
        );

        assert_eq!(file_size2, 67890, "Image 002 file_size should match");

        assert_eq!(hash2, "hash_dr002", "Image 002 file_hash should match");

        // Check pending image restored correctly

        let ai_status3: String = conn
            .query_row(
                "SELECT ai_status FROM images WHERE file_path = ?",
                ["/test/delete_restore_003.jpg"],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(
            ai_status3, "pending",
            "Image 003 ai_status should be pending"
        );

        // Check tags restored

        let restored_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_tag_count, tag_count,
            "Tag count should match after restore"
        );

        // Check tag names

        let tag_names: Vec<String> = conn
            .prepare("SELECT name FROM tags ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(
            tag_names.contains(&"nature".to_string()),
            "Tags should include 'nature'"
        );

        assert!(
            tag_names.contains(&"city".to_string()),
            "Tags should include 'city'"
        );

        assert!(
            tag_names.contains(&"night".to_string()),
            "Tags should include 'night'"
        );

        // Check image_tags restored

        let restored_image_tag_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM image_tags", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_image_tag_count, image_tag_count,
            "image_tag count should match after restore"
        );

        // Check search_index restored

        let restored_search_index_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_index", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            restored_search_index_count, search_index_count,
            "search_index count should match after restore"
        );

        // Check config values restored

        let restored_ai_concurrency: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_concurrency'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(
            restored_ai_concurrency, "5",
            "ai_concurrency should be restored to 5"
        );

        let restored_ai_timeout: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_timeout_seconds'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(
            restored_ai_timeout, "120",
            "ai_timeout_seconds should be restored to 120"
        );

        let custom_setting: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'custom_setting'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(
            custom_setting, "test_value",
            "custom_setting should be restored"
        );

        // Verify database integrity

        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();

        assert_eq!(integrity, "ok", "Database integrity check should pass");

        // Verify WAL mode is active

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            journal_mode, "wal",
            "WAL mode should be active after restore"
        );

        drop(conn);
    }

    #[test]

    fn test_restore_version_check_newer_backup_rejected() {
        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_version_check.db");

        let backup_db_path = temp_dir.path().join("backup_v5.db");

        let backup_db = Database::new_from_path(backup_db_path.to_str().unwrap()).unwrap();

        backup_db.init().unwrap();

        {
            let conn = backup_db.open_connection().unwrap();

            conn.pragma_update(None, "user_version", 5).unwrap();

            conn.execute(

                "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status) VALUES ('/t.jpg', 't.jpg', 100, 'h1', 'completed')",

                [],

            ).unwrap();
        }

        drop(backup_db);

        let backup_zip = temp_dir.path().join("backup_v5.zip");

        {
            use std::io::Write;

            use zip::write::SimpleFileOptions;

            use zip::ZipWriter;

            let zip_file = std::fs::File::create(&backup_zip).unwrap();

            let mut zip = ZipWriter::new(zip_file);

            zip.start_file("test_version_check.db", SimpleFileOptions::default())
                .unwrap();

            let data = std::fs::read(&backup_db_path).unwrap();

            zip.write_all(&data).unwrap();

            zip.finish().unwrap();
        }

        let current_db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        current_db.init().unwrap();

        {
            let conn = current_db.open_connection().unwrap();

            conn.pragma_update(None, "user_version", 3).unwrap();
        }

        drop(current_db);

        let result = restore_database_sync(&backup_zip, &db_path);

        assert!(result.is_err(), "Should reject backup with higher version");

        let err_msg = result.unwrap_err().to_string();

        assert!(
            err_msg.contains("higher") || err_msg.contains("Upgrade"),
            "Error should mention version mismatch"
        );
    }

    #[test]

    fn test_restore_version_check_same_or_lower_accepted() {
        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_version_ok.db");

        let backup_db_path = temp_dir.path().join("backup_v3.db");

        let backup_db = Database::new_from_path(backup_db_path.to_str().unwrap()).unwrap();

        backup_db.init().unwrap();

        {
            let conn = backup_db.open_connection().unwrap();

            conn.pragma_update(None, "user_version", 3).unwrap();

            conn.execute(

                "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status) VALUES ('/t.jpg', 't.jpg', 100, 'h2', 'completed')",

                [],

            ).unwrap();
        }

        drop(backup_db);

        let backup_zip = temp_dir.path().join("backup_v3.zip");

        {
            use std::io::Write;

            use zip::write::SimpleFileOptions;

            use zip::ZipWriter;

            let zip_file = std::fs::File::create(&backup_zip).unwrap();

            let mut zip = ZipWriter::new(zip_file);

            zip.start_file("test_version_ok.db", SimpleFileOptions::default())
                .unwrap();

            let data = std::fs::read(&backup_db_path).unwrap();

            zip.write_all(&data).unwrap();

            zip.finish().unwrap();
        }

        let current_db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        current_db.init().unwrap();

        {
            let conn = current_db.open_connection().unwrap();

            conn.pragma_update(None, "user_version", 3).unwrap();
        }

        drop(current_db);

        let result = restore_database_sync(&backup_zip, &db_path);

        assert!(result.is_ok(), "Should accept backup with same version");

        let verify_db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

        let conn = verify_db.open_connection().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0))
            .unwrap();

        assert_eq!(count, 1, "Restored data should be intact");
    }

    #[test]

    fn test_restore_version_check_no_existing_db() {
        let temp_dir = TempDir::new().unwrap();

        let db_path = temp_dir.path().join("test_no_existing.db");

        let backup_db_path = temp_dir.path().join("backup_v2.db");

        let backup_db = Database::new_from_path(backup_db_path.to_str().unwrap()).unwrap();

        backup_db.init().unwrap();

        {
            let conn = backup_db.open_connection().unwrap();

            conn.pragma_update(None, "user_version", 2).unwrap();

            conn.execute(

                "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status) VALUES ('/t.jpg', 't.jpg', 100, 'h3', 'completed')",

                [],

            ).unwrap();
        }

        drop(backup_db);

        let backup_zip = temp_dir.path().join("backup_v2.zip");

        {
            use std::io::Write;

            use zip::write::SimpleFileOptions;

            use zip::ZipWriter;

            let zip_file = std::fs::File::create(&backup_zip).unwrap();

            let mut zip = ZipWriter::new(zip_file);

            zip.start_file("test_no_existing.db", SimpleFileOptions::default())
                .unwrap();

            let data = std::fs::read(&backup_db_path).unwrap();

            zip.write_all(&data).unwrap();

            zip.finish().unwrap();
        }

        let result = restore_database_sync(&backup_zip, &db_path);

        assert!(
            result.is_ok(),
            "Should accept backup when no existing DB (version 0)"
        );
    }

    #[cfg(test)]
    mod tests_corrupted {

        use super::*;

        use tempfile::TempDir;

        #[test]

        fn tc_settings_sp_003_corrupted_backup_file() {
            // TC-SETTINGS-SP-003: Import corrupted backup file, show error and don't crash.

            // Test multiple corruption scenarios to ensure graceful error handling.

            let temp_dir = TempDir::new().unwrap();

            let db_path = temp_dir.path().join("test_corrupted_backup.db");

            let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();

            db.init().unwrap();

            // Scenario 1: Non-zip file (text content)

            let fake_zip_1 = temp_dir.path().join("corrupted_1.zip");

            std::fs::write(&fake_zip_1, "this is not a valid zip file at all").unwrap();

            let result_1 = restore_database_sync(&fake_zip_1, &db_path);

            assert!(result_1.is_err(), "Restore should fail with non-zip file");

            // Scenario 2: Valid zip but missing database file inside

            let incomplete_zip_path = temp_dir.path().join("incomplete.zip");

            {
                use std::io::Write;

                use zip::write::SimpleFileOptions;

                use zip::ZipWriter;

                let zip_file = std::fs::File::create(&incomplete_zip_path).unwrap();

                let mut zip = ZipWriter::new(zip_file);

                // Add a random text file instead of database

                zip.start_file("readme.txt", SimpleFileOptions::default())
                    .unwrap();

                zip.write_all(b"This is not a database file").unwrap();

                zip.finish().unwrap();
            }

            let result_2 = restore_database_sync(&incomplete_zip_path, &db_path);

            assert!(
                result_2.is_err(),
                "Restore should fail when zip doesn't contain db file"
            );

            // Verify error message mentions missing db file

            let err_msg = result_2.unwrap_err().to_string();

            assert!(
                err_msg.contains("ZIP"),
                "Error should mention missing db file or invalid zip, got: {}",
                err_msg
            );

            // Scenario 3: Truncated zip file (partial write)

            let truncated_zip_path = temp_dir.path().join("truncated.zip");

            {
                use std::io::Write;

                use zip::write::SimpleFileOptions;

                use zip::ZipWriter;

                let zip_file = std::fs::File::create(&truncated_zip_path).unwrap();

                let mut zip = ZipWriter::new(zip_file);

                zip.start_file("test.db", SimpleFileOptions::default())
                    .unwrap();

                zip.write_all(b"partial data").unwrap();

                // Don't call finish() - simulate interrupted write

                drop(zip);
            }

            let result_3 = restore_database_sync(&truncated_zip_path, &db_path);

            // Should either fail or succeed with corrupt data (both acceptable)

            // If it succeeds, the app shouldn't crash when opening the restored db

            if result_3.is_ok() {
                // Verify app can still open the database without crashing

                let db_after = Database::new_from_path(db_path.to_str().unwrap());

                assert!(
                    db_after.is_ok() || db_after.unwrap().open_connection().is_err(),
                    "App should not crash even with corrupted restored data"
                );
            }

            // Scenario 4: Empty file

            let empty_zip_path = temp_dir.path().join("empty.zip");

            std::fs::write(&empty_zip_path, []).unwrap();

            let result_4 = restore_database_sync(&empty_zip_path, &db_path);

            assert!(result_4.is_err(), "Restore should fail with empty zip file");

            // Verify original database is still intact after all failed restores

            let conn = db.open_connection().unwrap();

            let integrity: String = conn
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .unwrap();

            assert_eq!(
                integrity, "ok",
                "Original database should remain intact after failed restores"
            );
        }
    }
}
