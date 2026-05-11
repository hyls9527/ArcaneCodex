use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::{
    clip_embedder::ClipEmbedder,
    face_detector::FaceDetector,
    image_classifier::ImageClassifier,
    onnx_runtime::{ModelStatus, ModelType, OnnxRuntimeManager},
    vector_index::{HnswVectorIndex, VectorEntry},
};

pub struct AppState {
    pub onnx_manager: Arc<OnnxRuntimeManager>,
    pub classifier: Arc<ImageClassifier>,
    pub face_detector: Arc<FaceDetector>,
    pub clip_embedder: Arc<ClipEmbedder>,
    pub vector_index: Arc<HnswVectorIndex>,
}

#[derive(Debug, Serialize)]
pub struct ModelLoadResult {
    pub success: bool,
    pub model: Option<ModelStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_ai_model_status(state: State<'_, AppState>) -> Result<Vec<ModelStatus>, String> {
    Ok(state.onnx_manager.get_model_status().await)
}

#[tauri::command]
pub async fn load_ai_model(
    state: State<'_, AppState>,
    model_type: String,
    custom_path: Option<String>,
) -> Result<ModelLoadResult, String> {
    let model_type = match model_type.as_str() {
        "image_classification" => ModelType::ImageClassification,
        "face_detection" => ModelType::FaceDetection,
        "face_recognition" => ModelType::FaceRecognition,
        "clip_embedding" => ModelType::ClipEmbedding,
        _ => {
            return Ok(ModelLoadResult {
                success: false,
                model: None,
                error: Some(format!("未知模型类型: {}", model_type)),
            });
        }
    };

    let path = custom_path.as_ref().map(PathBuf::from);

    match state
        .onnx_manager
        .load_model(model_type, path.as_deref())
        .await
    {
        Ok(config) => {
            let _ = state.onnx_manager.warmup_model(model_type).await;
            Ok(ModelLoadResult {
                success: true,
                model: Some(ModelStatus {
                    model_type: config.model_type.as_str().to_string(),
                    name: config.name,
                    is_loaded: true,
                    path: config.path.to_string_lossy().to_string(),
                }),
                error: None,
            })
        }
        Err(e) => Ok(ModelLoadResult {
            success: false,
            model: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn unload_ai_model(
    state: State<'_, AppState>,
    model_type: String,
) -> Result<bool, String> {
    let model_type = match model_type.as_str() {
        "image_classification" => ModelType::ImageClassification,
        "face_detection" => ModelType::FaceDetection,
        "face_recognition" => ModelType::FaceRecognition,
        "clip_embedding" => ModelType::ClipEmbedding,
        _ => return Err(format!("未知模型类型: {}", model_type)),
    };

    Ok(state.onnx_manager.unload_model(model_type).await)
}

#[tauri::command]
pub async fn classify_image(
    state: State<'_, AppState>,
    image_path: String,
    top_n: usize,
) -> Result<crate::core::image_classifier::ImageClassificationResult, String> {
    let path = PathBuf::from(&image_path);
    state
        .classifier
        .classify_image(&path, top_n)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct BoundingBoxDto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<BoundingBoxDto> for crate::core::face_detector::BoundingBox {
    fn from(dto: BoundingBoxDto) -> Self {
        Self {
            x: dto.x,
            y: dto.y,
            width: dto.width,
            height: dto.height,
        }
    }
}

#[tauri::command]
pub async fn detect_faces(
    state: State<'_, AppState>,
    image_path: String,
    confidence_threshold: f32,
) -> Result<Vec<crate::core::face_detector::FaceDetection>, String> {
    let path = PathBuf::from(&image_path);
    state
        .face_detector
        .detect_faces(&path, confidence_threshold)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_face_embedding(
    state: State<'_, AppState>,
    image_path: String,
    bbox: BoundingBoxDto,
) -> Result<crate::core::face_detector::FaceEmbedding, String> {
    let path = PathBuf::from(&image_path);
    let bbox_internal: crate::core::face_detector::BoundingBox = bbox.into();
    state
        .face_detector
        .extract_face_embedding(&path, &bbox_internal)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register_face(
    state: State<'_, AppState>,
    image_path: String,
    bbox: BoundingBoxDto,
) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    let bbox_internal: crate::core::face_detector::BoundingBox = bbox.into();
    state
        .face_detector
        .register_face(&path, &bbox_internal)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn recognize_face(
    state: State<'_, AppState>,
    image_path: String,
    bbox: BoundingBoxDto,
    threshold: f32,
) -> Result<Option<crate::core::face_detector::FaceMatch>, String> {
    let path = PathBuf::from(&image_path);
    let bbox_internal: crate::core::face_detector::BoundingBox = bbox.into();
    state
        .face_detector
        .recognize_face(&path, &bbox_internal, threshold)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_registered_face_count(state: State<'_, AppState>) -> Result<usize, String> {
    Ok(state.face_detector.get_registered_count().await)
}

#[tauri::command]
pub async fn embed_image_clip(
    state: State<'_, AppState>,
    image_path: String,
) -> Result<crate::core::clip_embedder::ClipEmbedding, String> {
    let path = PathBuf::from(&image_path);
    state
        .clip_embedder
        .embed_image(&path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct VectorEntryDto {
    pub id: String,
    pub embedding: Vec<f32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub image_path: Option<String>,
}

impl From<VectorEntryDto> for VectorEntry {
    fn from(dto: VectorEntryDto) -> Self {
        Self {
            id: dto.id,
            embedding: dto.embedding,
            metadata: dto.metadata,
            image_path: dto.image_path,
            created_at: chrono::Utc::now(),
        }
    }
}

#[tauri::command]
pub async fn insert_vector(
    state: State<'_, AppState>,
    entry: VectorEntryDto,
) -> Result<(), String> {
    let entry_internal: VectorEntry = entry.into();
    state
        .vector_index
        .insert(entry_internal)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_vectors(
    state: State<'_, AppState>,
    query: Vec<f32>,
    top_k: usize,
    min_similarity: f32,
) -> Result<Vec<crate::core::vector_index::SearchResult>, String> {
    state
        .vector_index
        .search(&query, top_k, min_similarity)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_vector(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    Ok(state.vector_index.delete(&id).await)
}

#[tauri::command]
pub async fn get_vector_index_stats(
    state: State<'_, AppState>,
) -> Result<crate::core::vector_index::IndexStats, String> {
    Ok(state.vector_index.get_stats().await)
}
