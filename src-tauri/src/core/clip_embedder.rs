#![allow(missing_docs)]
use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::core::onnx_runtime::{ModelType, OnnxRuntimeManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEmbedding {
    pub image_id: String,
    pub embedding: Vec<f32>,
    pub image_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub enum ClipError {
    ModelNotLoaded,
    ImageProcessingFailed(String),
    InferenceFailed(String),
    InvalidEmbeddingDimension(usize),
}

impl std::fmt::Display for ClipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipError::ModelNotLoaded => write!(f, "CLIP模型未加载"),
            ClipError::ImageProcessingFailed(msg) => write!(f, "图像处理失败: {}", msg),
            ClipError::InferenceFailed(msg) => write!(f, "推理失败: {}", msg),
            ClipError::InvalidEmbeddingDimension(dim) => {
                write!(f, "无效的嵌入维度: {}", dim)
            }
        }
    }
}

impl From<image::ImageError> for ClipError {
    fn from(e: image::ImageError) -> Self {
        ClipError::ImageProcessingFailed(e.to_string())
    }
}

pub type ClipResult<T> = std::result::Result<T, ClipError>;

const CLIP_MEAN: [f64; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f64; 3] = [0.26862954, 0.26130258, 0.27577711];

pub struct ClipEmbedder {
    runtime_manager: Arc<OnnxRuntimeManager>,
    expected_dim: usize,
}

impl ClipEmbedder {
    pub fn new(runtime_manager: Arc<OnnxRuntimeManager>) -> Self {
        tracing::info!("初始化CLIP嵌入器");
        Self {
            runtime_manager,
            expected_dim: 512,
        }
    }

    pub async fn embed_image(&self, image_path: &Path) -> ClipResult<ClipEmbedding> {
        if !self
            .runtime_manager
            .is_model_loaded(ModelType::ClipEmbedding)
            .await
        {
            return Err(ClipError::ModelNotLoaded);
        }

        let img = image::open(image_path)?;
        let input_data = Self::preprocess_image(&img)?;

        let result = self
            .runtime_manager
            .run_inference(ModelType::ClipEmbedding, &input_data, vec![1, 3, 224, 224])
            .await
            .map_err(|e| ClipError::InferenceFailed(e.to_string()))?;

        let embedding = self.extract_embedding(&result)?;

        if embedding.len() != self.expected_dim {
            return Err(ClipError::InvalidEmbeddingDimension(embedding.len()));
        }

        Ok(ClipEmbedding {
            image_id: uuid::Uuid::new_v4().to_string(),
            embedding,
            image_path: image_path.to_path_buf(),
            created_at: chrono::Utc::now(),
        })
    }

    fn preprocess_image(img: &DynamicImage) -> ClipResult<Vec<f32>> {
        let resized = img.resize_exact(224, 224, image::imageops::FilterType::Lanczos3);

        let rgb = resized.to_rgb8();
        let mut normalized_data = Vec::with_capacity(224 * 224 * 3);

        for pixel in rgb.pixels() {
            normalized_data.push(((pixel[0] as f64 / 255.0 - CLIP_MEAN[0]) / CLIP_STD[0]) as f32);
            normalized_data.push(((pixel[1] as f64 / 255.0 - CLIP_MEAN[1]) / CLIP_STD[1]) as f32);
            normalized_data.push(((pixel[2] as f64 / 255.0 - CLIP_MEAN[2]) / CLIP_STD[2]) as f32);
        }

        Ok(normalized_data)
    }

    fn extract_embedding(
        &self,
        result: &crate::core::onnx_runtime::InferenceResult,
    ) -> ClipResult<Vec<f32>> {
        if let Some((_, values)) = result.outputs.iter().next() {
            return Ok(values.clone());
        }

        Err(ClipError::InferenceFailed("无法提取嵌入向量".to_string()))
    }

}
