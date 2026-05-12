#![allow(missing_docs)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::core::onnx_runtime::{ModelType, OnnxRuntimeManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetection {
    pub bbox: BoundingBox,
    pub landmarks: Vec<Landmark>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landmark {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceEmbedding {
    pub face_id: String,
    pub embedding: Vec<f32>,
    pub image_path: PathBuf,
    pub detection: FaceDetection,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatch {
    pub face_id: String,
    pub similarity: f32,
    pub face_embedding: FaceEmbedding,
}

#[derive(Debug)]
pub enum FaceError {
    ModelNotLoaded(String),
    ImageProcessingFailed(String),
    InferenceFailed(String),
    NoFacesDetected,
    #[allow(dead_code)]
    InvalidInput(String),
}

impl std::fmt::Display for FaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaceError::ModelNotLoaded(model) => write!(f, "人脸模型未加载: {}", model),
            FaceError::ImageProcessingFailed(msg) => write!(f, "图像处理失败: {}", msg),
            FaceError::InferenceFailed(msg) => write!(f, "推理失败: {}", msg),
            FaceError::NoFacesDetected => write!(f, "未检测到人脸"),
            FaceError::InvalidInput(msg) => write!(f, "无效输入: {}", msg),
        }
    }
}

impl From<image::ImageError> for FaceError {
    fn from(e: image::ImageError) -> Self {
        FaceError::ImageProcessingFailed(e.to_string())
    }
}

pub type FaceResult<T> = std::result::Result<T, FaceError>;

pub struct FaceDetector {
    runtime_manager: Arc<OnnxRuntimeManager>,
    face_embeddings: Arc<tokio::sync::RwLock<HashMap<String, FaceEmbedding>>>,
}

impl FaceDetector {
    pub fn new(runtime_manager: Arc<OnnxRuntimeManager>) -> Self {
        tracing::info!("初始化人脸检测器");
        Self {
            runtime_manager,
            face_embeddings: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn detect_faces(
        &self,
        image_path: &Path,
        confidence_threshold: f32,
    ) -> FaceResult<Vec<FaceDetection>> {
        if !self
            .runtime_manager
            .is_model_loaded(ModelType::FaceDetection)
            .await
        {
            return Err(FaceError::ModelNotLoaded("face_detection".to_string()));
        }

        let img = image::open(image_path)?;
        let (input_data, original_size) = Self::preprocess_for_detection(&img)?;

        let result = self
            .runtime_manager
            .run_inference(ModelType::FaceDetection, &input_data, vec![1, 3, 640, 640])
            .await
            .map_err(|e| FaceError::InferenceFailed(e.to_string()))?;

        let faces = self.parse_detection_result(&result, original_size, confidence_threshold)?;

        if faces.is_empty() {
            return Err(FaceError::NoFacesDetected);
        }

        Ok(faces)
    }

    pub async fn extract_face_embedding(
        &self,
        image_path: &Path,
        face_bbox: &BoundingBox,
    ) -> FaceResult<FaceEmbedding> {
        if !self
            .runtime_manager
            .is_model_loaded(ModelType::FaceRecognition)
            .await
        {
            return Err(FaceError::ModelNotLoaded("face_recognition".to_string()));
        }

        let img = image::open(image_path)?;
        let face_image = Self::crop_face(&img, face_bbox)?;
        let input_data = Self::preprocess_for_recognition(&face_image)?;

        let result = self
            .runtime_manager
            .run_inference(
                ModelType::FaceRecognition,
                &input_data,
                vec![1, 3, 112, 112],
            )
            .await
            .map_err(|e| FaceError::InferenceFailed(e.to_string()))?;

        let embedding = self.extract_embedding_from_result(&result)?;

        Ok(FaceEmbedding {
            face_id: uuid::Uuid::new_v4().to_string(),
            embedding,
            image_path: image_path.to_path_buf(),
            detection: FaceDetection {
                bbox: face_bbox.clone(),
                landmarks: Vec::new(),
                confidence: 0.0,
            },
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn register_face(
        &self,
        image_path: &Path,
        face_bbox: &BoundingBox,
    ) -> FaceResult<String> {
        let embedding = self.extract_face_embedding(image_path, face_bbox).await?;
        let face_id = embedding.face_id.clone();

        {
            let mut embeddings = self.face_embeddings.write().await;
            embeddings.insert(face_id.clone(), embedding);
        }

        tracing::info!(face_id = %face_id, path = %image_path.display(), "人脸注册成功");

        Ok(face_id)
    }

    pub async fn recognize_face(
        &self,
        image_path: &Path,
        face_bbox: &BoundingBox,
        threshold: f32,
    ) -> FaceResult<Option<FaceMatch>> {
        let query_embedding = self.extract_face_embedding(image_path, face_bbox).await?;

        let embeddings = self.face_embeddings.read().await;

        let mut best_match: Option<FaceMatch> = None;
        let mut best_similarity = 0.0f32;

        for (face_id, stored_embedding) in embeddings.iter() {
            let similarity =
                Self::cosine_similarity(&query_embedding.embedding, &stored_embedding.embedding);

            if similarity > best_similarity && similarity >= threshold {
                best_similarity = similarity;
                best_match = Some(FaceMatch {
                    face_id: face_id.clone(),
                    similarity,
                    face_embedding: stored_embedding.clone(),
                });
            }
        }

        drop(embeddings);

        Ok(best_match)
    }

    #[allow(dead_code)]
    pub async fn find_similar_faces(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        min_similarity: f32,
    ) -> FaceResult<Vec<FaceMatch>> {
        let embeddings = self.face_embeddings.read().await;

        let mut matches: Vec<FaceMatch> = Vec::new();

        for (face_id, stored_embedding) in embeddings.iter() {
            let similarity = Self::cosine_similarity(query_embedding, &stored_embedding.embedding);

            if similarity >= min_similarity {
                matches.push(FaceMatch {
                    face_id: face_id.clone(),
                    similarity,
                    face_embedding: stored_embedding.clone(),
                });
            }
        }

        drop(embeddings);

        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matches.into_iter().take(top_k).collect())
    }

    pub async fn get_registered_count(&self) -> usize {
        let embeddings = self.face_embeddings.read().await;
        embeddings.len()
    }

    #[allow(dead_code)]
    pub async fn unregister_face(&self, face_id: &str) -> bool {
        let mut embeddings = self.face_embeddings.write().await;
        embeddings.remove(face_id).is_some()
    }

    fn preprocess_for_detection(img: &DynamicImage) -> FaceResult<(Vec<f32>, (u32, u32))> {
        let original_size = (img.width(), img.height());
        let resized = img.resize_exact(640, 640, image::imageops::FilterType::Lanczos3);

        let rgb = resized.to_rgb8();
        let mut normalized_data = Vec::new();

        for pixel in rgb.pixels() {
            normalized_data.push(pixel[0] as f32 / 255.0);
            normalized_data.push(pixel[1] as f32 / 255.0);
            normalized_data.push(pixel[2] as f32 / 255.0);
        }

        Ok((normalized_data, original_size))
    }

    fn crop_face(img: &DynamicImage, bbox: &BoundingBox) -> FaceResult<DynamicImage> {
        let x = bbox.x.max(0.0) as u32;
        let y = bbox.y.max(0.0) as u32;
        let width = (bbox.width + bbox.x - x as f32).max(1.0) as u32;
        let height = (bbox.height + bbox.y - y as f32).max(1.0) as u32;

        let cropped = image::imageops::crop(&mut img.to_rgba8(), x, y, width, height).to_image();
        Ok(DynamicImage::ImageRgba8(cropped))
    }

    fn preprocess_for_recognition(face_img: &DynamicImage) -> FaceResult<Vec<f32>> {
        let resized = face_img.resize_exact(112, 112, image::imageops::FilterType::Lanczos3);

        let rgb = resized.to_rgb8();
        let mut normalized_data = Vec::new();

        for pixel in rgb.pixels() {
            normalized_data.push((pixel[0] as f32 - 127.5) / 128.0);
            normalized_data.push((pixel[1] as f32 - 127.5) / 128.0);
            normalized_data.push((pixel[2] as f32 - 127.5) / 128.0);
        }

        Ok(normalized_data)
    }

    fn parse_detection_result(
        &self,
        result: &crate::core::onnx_runtime::InferenceResult,
        original_size: (u32, u32),
        confidence_threshold: f32,
    ) -> FaceResult<Vec<FaceDetection>> {
        let mut faces = Vec::new();

        if let Some((_, values)) = result.outputs.iter().next() {
            let num_detections = values.len() / 15;

            for i in 0..num_detections {
                let base_idx = i * 15;
                if base_idx + 14 < values.len() {
                    let confidence = values[base_idx + 14];

                    if confidence >= confidence_threshold {
                        let scale_x = original_size.0 as f32 / 640.0;
                        let scale_y = original_size.1 as f32 / 640.0;

                        let bbox = BoundingBox {
                            x: values[base_idx] * scale_x,
                            y: values[base_idx + 1] * scale_y,
                            width: (values[base_idx + 2] - values[base_idx]) * scale_x,
                            height: (values[base_idx + 3] - values[base_idx + 1]) * scale_y,
                        };

                        let mut landmarks = Vec::new();
                        for j in 0..5 {
                            landmarks.push(Landmark {
                                x: values[base_idx + 4 + j * 2] * scale_x,
                                y: values[base_idx + 5 + j * 2] * scale_y,
                            });
                        }

                        faces.push(FaceDetection {
                            bbox,
                            landmarks,
                            confidence,
                        });
                    }
                }
            }
        }

        Ok(faces)
    }

    fn extract_embedding_from_result(
        &self,
        result: &crate::core::onnx_runtime::InferenceResult,
    ) -> FaceResult<Vec<f32>> {
        if let Some((_, values)) = result.outputs.iter().next() {
            return Ok(values.clone());
        }

        Err(FaceError::InferenceFailed("无法提取嵌入向量".to_string()))
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}
