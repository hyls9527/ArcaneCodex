// Tauri 2.x Desktop Application - Arcane Codex
// Local-first Image Knowledge Base

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod core;
mod models;
mod utils;

use std::sync::Arc;
use tauri::Manager;

fn main() {
    // 初始化日志系统
    utils::error::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::images::import_images,
            commands::images::get_images,
            commands::images::get_image_detail,
            commands::images::delete_images,
            commands::images::check_broken_links,
            commands::images::archive_image,
            commands::images::safe_export,
            commands::ai::start_ai_processing,
            commands::ai::pause_ai_processing,
            commands::ai::resume_ai_processing,
            commands::ai::get_ai_status,
            commands::ai::retry_failed_ai,
            commands::ai::get_recent_ai_results,
            commands::search::semantic_search,
            commands::dedup::scan_duplicates,
            commands::dedup::delete_duplicates,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::get_all_configs,
            commands::settings::backup_database,
            commands::settings::restore_database,
            commands::settings::test_lm_studio_connection,
            commands::inference_settings::get_inference_config,
            commands::inference_settings::set_inference_provider,
            commands::inference_settings::test_inference_connection,
            commands::inference_settings::discover_available_models,
            commands::export::export_data,
            commands::narrative::write_narrative,
            commands::narrative::get_narratives,
            commands::narrative::query_associations,
            commands::tag_correction::record_tag_correction,
            commands::tag_correction::get_tag_correction_history,
            commands::tag_correction::get_all_tag_corrections,
            commands::error_patterns::record_error_pattern,
            commands::error_patterns::get_error_patterns,
            commands::error_patterns::check_error_pattern_exists,
            commands::error_patterns::delete_error_pattern,
            commands::error_patterns::get_high_frequency_error_patterns,
            commands::batch_ops::start_batch_ai_tag,
            commands::batch_ops::get_batch_ai_status,
            commands::batch_ops::pause_batch_ai_task,
            commands::batch_ops::resume_batch_ai_task,
            commands::batch_ops::cancel_batch_ai_task,
            commands::batch_ops::batch_tag_correction,
            commands::batch_ops::batch_export,
            commands::batch_ops::get_library_stats,
            commands::batch_ops::get_accuracy_trend,
            commands::batch_ops::get_log_entries,
            commands::batch_ops::get_log_stats,
            commands::batch_ops::export_logs,
            commands::batch_ops::clear_logs,
            commands::seed_data::check_sample_data,
            commands::seed_data::clear_sample_data,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            let db = core::db::Database::new(app_handle).map_err(|e| {
                tauri::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;
            db.run_migrations().map_err(|e| {
                tauri::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            if let Err(e) = commands::seed_data::seed_if_empty(&db) {
                tracing::warn!("Failed to seed sample data: {}", e);
            }

            app.manage(db.clone());

            let queue = core::ai_queue::AITaskQueue::new(Arc::new(db), None)
                .with_app_handle(app_handle.clone());
            app.manage(Arc::new(queue));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
