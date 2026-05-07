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
use crate::core::file_watcher::FileWatcherService;
use crate::core::onnx_runtime::OnnxRuntimeManager;
use crate::core::image_classifier::ImageClassifier;
use crate::core::face_detector::FaceDetector;
use crate::core::clip_embedder::ClipEmbedder;
use crate::core::vector_index::HnswVectorIndex;
use crate::core::knowledge_graph::KnowledgeGraphEngine;

fn main() {
    // 初始化日志系统
    utils::error::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::settings::backup_database_encrypted,
            commands::settings::restore_database,
            commands::settings::restore_database_encrypted,
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
            commands::xmp::read_xmp_metadata,
            commands::xmp::write_xmp_metadata,
            commands::xmp::generate_xmp_sidecar,
            commands::xmp::export_as_xmp,
            commands::seed_data::check_sample_data,
            commands::seed_data::clear_sample_data,
            commands::seed_data::load_sample_data,
            commands::file_monitor::start_file_monitor,
            commands::file_monitor::stop_file_monitor,
            commands::file_monitor::get_monitor_status,
            // AI Core Commands (ONNX Runtime, Classification, Face Detection, CLIP, Vector Search)
            commands::ai_core::get_ai_model_status,
            commands::ai_core::load_ai_model,
            commands::ai_core::unload_ai_model,
            commands::ai_core::classify_image,
            commands::ai_core::detect_faces,
            commands::ai_core::extract_face_embedding,
            commands::ai_core::register_face,
            commands::ai_core::recognize_face,
            commands::ai_core::get_registered_face_count,
            commands::ai_core::embed_image_clip,
            commands::ai_core::insert_vector,
            commands::ai_core::search_vectors,
            commands::ai_core::delete_vector,
            commands::ai_core::get_vector_index_stats,
            // Knowledge Graph Commands
            commands::knowledge_graph::kg_build_graph,
            commands::knowledge_graph::kg_get_stats,
            commands::knowledge_graph::kg_get_all_nodes,
            commands::knowledge_graph::kg_get_all_edges,
            commands::knowledge_graph::kg_get_communities,
            commands::knowledge_graph::kg_get_community_nodes,
            commands::knowledge_graph::kg_get_neighbors,
            commands::knowledge_graph::kg_find_path,
            commands::knowledge_graph::kg_search_nodes,
            commands::knowledge_graph::kg_clear,
            commands::knowledge_graph::kg_load_from_db,
            commands::knowledge_graph::kg_save_to_db,
            // Calibration Commands
            commands::calibration::record_calibration_sample,
            commands::calibration::calculate_and_save_calibration,
            commands::calibration::get_latest_calibration_report,
            commands::calibration::get_calibration_curve_data,
        ])
        .setup(|app| {
            let (tx, _rx) = tokio::sync::broadcast::channel(256);
            app.manage(commands::file_monitor::MonitorState {
                service: Arc::new(std::sync::Mutex::new(FileWatcherService::new(tx))),
            });

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

            let db_for_queue = db.clone();
            let queue = core::ai_queue::AITaskQueue::new(Arc::new(db_for_queue), None)
                .with_app_handle(app_handle.clone());
            app.manage(Arc::new(queue));

            // Initialize AI Core services
            let models_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("models");

            if !models_dir.exists() {
                std::fs::create_dir_all(&models_dir).ok();
            }

            let vector_index_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("vector_index");

            if !vector_index_dir.exists() {
                std::fs::create_dir_all(&vector_index_dir).ok();
            }

            let onnx_manager = Arc::new(OnnxRuntimeManager::new(&models_dir));
            let classifier = Arc::new(ImageClassifier::new(onnx_manager.clone()));
            let face_detector = Arc::new(FaceDetector::new(onnx_manager.clone()));
            let clip_embedder = Arc::new(ClipEmbedder::new(onnx_manager.clone()));
            let vector_index = Arc::new(HnswVectorIndex::new(512, &vector_index_dir));

            app.manage(commands::ai_core::AppState {
                onnx_manager,
                classifier,
                face_detector,
                clip_embedder: clip_embedder.clone(),
                vector_index: vector_index.clone(),
            });

            let kg_engine = Arc::new(KnowledgeGraphEngine::new(
                Arc::new(db.clone()),
                clip_embedder,
                vector_index,
            ));
            app.manage(commands::knowledge_graph::KgState {
                engine: kg_engine,
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
