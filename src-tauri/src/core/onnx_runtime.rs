#![allow(missing_docs)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ort::session::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    ImageClassification,
    FaceDetection,
    FaceRecognition,
    ClipEmbedding,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::ImageClassification => "image_classification",
            ModelType::FaceDetection => "face_detection",
            ModelType::FaceRecognition => "face_recognition",
            ModelType::ClipEmbedding => "clip_embedding",
        }
    }

    pub fn default_model_name(&self) -> &'static str {
        match self {
            ModelType::ImageClassification => "mobilenetv3_large.onnx",
            ModelType::FaceDetection => "retinaface_resnet50.onnx",
            ModelType::FaceRecognition => "arcface_r100.onnx",
            ModelType::ClipEmbedding => "clip_vit_b32.onnx",
        }
    }

    #[allow(dead_code)]
    pub fn input_shape(&self) -> Vec<usize> {
        match self {
            ModelType::ImageClassification => vec![1, 3, 224, 224],
            ModelType::FaceDetection => vec![1, 3, 640, 640],
            ModelType::FaceRecognition => vec![1, 3, 112, 112],
            ModelType::ClipEmbedding => vec![1, 3, 224, 224],
        }
    }

    #[allow(dead_code)]
    pub fn output_dim(&self) -> usize {
        match self {
            ModelType::ImageClassification => 1000,
            ModelType::FaceDetection => 42000,
            ModelType::FaceRecognition => 512,
            ModelType::ClipEmbedding => 512,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: ModelType,
    pub name: String,
    pub path: PathBuf,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub is_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub outputs: HashMap<String, Vec<f32>>,
    pub inference_time_ms: u64,
    pub model_type: String,
}

#[derive(Debug)]
pub enum OnnxError {
    SessionCreate(String),
    ModelNotFound(PathBuf),
    InvalidInput(String),
    InferenceFailed(String),
    Io(std::io::Error),
    Ort(ort::Error),
}

impl std::fmt::Display for OnnxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnnxError::SessionCreate(msg) => write!(f, "创建ONNX会话失败: {}", msg),
            OnnxError::ModelNotFound(path) => write!(f, "模型文件不存在: {}", path.display()),
            OnnxError::InvalidInput(msg) => write!(f, "无效的输入数据: {}", msg),
            OnnxError::InferenceFailed(msg) => write!(f, "推理失败: {}", msg),
            OnnxError::Io(e) => write!(f, "IO错误: {}", e),
            OnnxError::Ort(e) => write!(f, "ONNX运行时错误: {}", e),
        }
    }
}

impl From<ort::Error> for OnnxError {
    fn from(e: ort::Error) -> Self {
        OnnxError::Ort(e)
    }
}

impl From<std::io::Error> for OnnxError {
    fn from(e: std::io::Error) -> Self {
        OnnxError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, OnnxError>;

pub struct OnnxRuntimeManager {
    sessions: Arc<tokio::sync::RwLock<HashMap<ModelType, Session>>>,
    configs: Arc<tokio::sync::RwLock<HashMap<ModelType, ModelConfig>>>,
    models_dir: PathBuf,
}

impl OnnxRuntimeManager {
    pub fn new(models_dir: &Path) -> Self {
        tracing::info!(path = %models_dir.display(), "初始化ONNX Runtime管理器");
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            configs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            models_dir: models_dir.to_path_buf(),
        }
    }

    pub async fn load_model(
        &self,
        model_type: ModelType,
        custom_path: Option<&Path>,
    ) -> Result<ModelConfig> {
        let model_path = if let Some(path) = custom_path {
            path.to_path_buf()
        } else {
            self.models_dir.join(model_type.default_model_name())
        };

        if !model_path.exists() {
            return Err(OnnxError::ModelNotFound(model_path.clone()));
        }

        let session = Session::builder()
            .map_err(|e| OnnxError::SessionCreate(format!("构建配置失败: {}", e)))?
            .commit_from_file(&model_path)
            .map_err(|e| OnnxError::SessionCreate(format!("加载模型失败: {}", e)))?;

        let input_names: Vec<String> = session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let output_names: Vec<String> = session
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();

        let config = ModelConfig {
            model_type,
            name: model_type.default_model_name().to_string(),
            path: model_path.clone(),
            input_names: input_names.clone(),
            output_names: output_names.clone(),
            is_loaded: true,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(model_type, session);
        }

        {
            let mut configs = self.configs.write().await;
            configs.insert(model_type, config.clone());
        }

        tracing::info!(
            model = model_type.as_str(),
            path = %model_path.display(),
            inputs = ?input_names,
            outputs = ?output_names,
            "模型加载成功"
        );

        Ok(config)
    }

    pub async fn unload_model(&self, model_type: ModelType) -> bool {
        let removed = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&model_type).is_some()
        };

        if removed {
            let mut configs = self.configs.write().await;
            if let Some(config) = configs.get_mut(&model_type) {
                config.is_loaded = false;
            }
            tracing::info!(model = model_type.as_str(), "模型已卸载");
        }

        removed
    }

    pub async fn run_inference(
        &self,
        model_type: ModelType,
        input_data: &[f32],
        input_shape: Vec<usize>,
    ) -> Result<InferenceResult> {
        let config = {
            let configs = self.configs.read().await;
            configs.get(&model_type).cloned().ok_or_else(|| {
                OnnxError::InferenceFailed(format!("模型未加载: {:?}", model_type))
            })?
        };

        let start_time = std::time::Instant::now();

        let input_tensor =
            ort::value::Tensor::from_array((input_shape.clone(), input_data.to_vec()))
                .map_err(|e| OnnxError::InvalidInput(format!("创建张量失败: {}", e)))?;

        let output_data = {
            let mut sessions = self.sessions.write().await;
            // NOTE: write lock required — ort::Session::run() takes &mut self
            if let Some(session) = sessions.get_mut(&model_type) {
                let outputs = session
                    .run(ort::inputs![input_tensor])
                    .map_err(|e| OnnxError::InferenceFailed(format!("执行推理失败: {}", e)))?;

                let mut data_map = HashMap::new();
                for (name, output) in outputs.iter() {
                    if config.output_names.contains(&name.to_string()) {
                        if let Ok(tensor) = output.try_extract_tensor::<f32>() {
                            data_map.insert(name.to_string(), tensor.1.to_vec());
                        }
                    }
                }

                Ok(data_map)
            } else {
                Err(OnnxError::InferenceFailed(format!(
                    "模型未加载: {:?}",
                    model_type
                )))
            }
        }?;

        let inference_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            outputs: output_data,
            inference_time_ms,
            model_type: model_type.as_str().to_string(),
        })
    }

    pub async fn get_model_status(&self) -> Vec<ModelStatus> {
        let configs = self.configs.read().await;
        configs
            .values()
            .map(|c| ModelStatus {
                model_type: c.model_type.as_str().to_string(),
                name: c.name.clone(),
                is_loaded: c.is_loaded,
                path: c.path.to_string_lossy().to_string(),
            })
            .collect()
    }

    pub async fn is_model_loaded(&self, model_type: ModelType) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(&model_type)
    }

    pub async fn warmup_model(&self, model_type: ModelType) -> Result<()> {
        let shape = model_type.input_shape();
        let total_elements: usize = shape.iter().product();
        let dummy_data = vec![0.0f32; total_elements];

        for i in 0..3 {
            match self
                .run_inference(model_type, &dummy_data, shape.clone())
                .await
            {
                Ok(_) => tracing::info!(model = model_type.as_str(), round = i + 1, "warmup 完成"),
                Err(e) => {
                    tracing::warn!(model = model_type.as_str(), round = i + 1, error = %e, "warmup 失败");
                }
            }
        }

        tracing::info!(model = model_type.as_str(), "模型预热完成");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_models_dir(&self) -> &Path {
        &self.models_dir
    }

    #[allow(dead_code)]
    pub async fn list_available_models(&self) -> Vec<PathBuf> {
        let mut models = Vec::new();

        if self.models_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("onnx") {
                        models.push(path);
                    }
                }
            }
        }

        models.sort();
        models
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub model_type: String,
    pub name: String,
    pub is_loaded: bool,
    pub path: String,
}
