use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
    pub image_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub similarity: f32,
    pub entry: VectorEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_size_bytes: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub enum VectorIndexError {
    #[allow(dead_code)]
    IndexNotInitialized,
    DimensionMismatch {
        expected: usize,
        got: usize,
    },
    EmptyVector,
    #[allow(dead_code)]
    InsertFailed(String),
    #[allow(dead_code)]
    SearchFailed(String),
    #[allow(dead_code)]
    SerializationError(String),
    #[allow(dead_code)]
    DeserializationError(String),
    Io(std::io::Error),
}

impl std::fmt::Display for VectorIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorIndexError::IndexNotInitialized => write!(f, "向量索引未初始化"),
            VectorIndexError::DimensionMismatch { expected, got } => {
                write!(f, "维度不匹配: 预期 {}, 实际 {}", expected, got)
            }
            VectorIndexError::EmptyVector => write!(f, "向量不能为空"),
            VectorIndexError::InsertFailed(msg) => write!(f, "插入失败: {}", msg),
            VectorIndexError::SearchFailed(msg) => write!(f, "搜索失败: {}", msg),
            VectorIndexError::SerializationError(msg) => {
                write!(f, "序列化失败: {}", msg)
            }
            VectorIndexError::DeserializationError(msg) => {
                write!(f, "反序列化失败: {}", msg)
            }
            VectorIndexError::Io(e) => write!(f, "IO错误: {}", e),
        }
    }
}

impl From<std::io::Error> for VectorIndexError {
    fn from(e: std::io::Error) -> Self {
        VectorIndexError::Io(e)
    }
}

pub type VectorResult<T> = std::result::Result<T, VectorIndexError>;

pub struct HnswVectorIndex {
    dimension: usize,
    entries: Arc<tokio::sync::RwLock<HashMap<String, VectorEntry>>>,
    #[allow(dead_code)]
    index_dir: PathBuf,
}

impl HnswVectorIndex {
    pub fn new(dimension: usize, index_dir: &Path) -> Self {
        tracing::info!(
            dimension = dimension,
            path = %index_dir.display(),
            "初始化HNSW向量索引"
        );
        Self {
            dimension,
            entries: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            index_dir: index_dir.to_path_buf(),
        }
    }

    pub async fn insert(&self, entry: VectorEntry) -> VectorResult<()> {
        if entry.embedding.is_empty() {
            return Err(VectorIndexError::EmptyVector);
        }

        if entry.embedding.len() != self.dimension {
            return Err(VectorIndexError::DimensionMismatch {
                expected: self.dimension,
                got: entry.embedding.len(),
            });
        }

        let mut entries = self.entries.write().await;
        entries.insert(entry.id.clone(), entry);

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn batch_insert(&self, entries: Vec<VectorEntry>) -> VectorResult<(usize, usize)> {
        let mut success_count = 0;
        let mut fail_count = 0;

        for entry in entries {
            match self.insert(entry).await {
                Ok(()) => success_count += 1,
                Err(_) => fail_count += 1,
            }
        }

        Ok((success_count, fail_count))
    }

    pub async fn search(
        &self,
        query: &[f32],
        top_k: usize,
        min_similarity: f32,
    ) -> VectorResult<Vec<SearchResult>> {
        if query.is_empty() {
            return Err(VectorIndexError::EmptyVector);
        }

        if query.len() != self.dimension {
            return Err(VectorIndexError::DimensionMismatch {
                expected: self.dimension,
                got: query.len(),
            });
        }

        let entries = self.entries.read().await;

        let mut results: Vec<SearchResult> = Vec::new();

        for (id, entry) in entries.iter() {
            let similarity = Self::cosine_similarity(query, &entry.embedding);

            if similarity >= min_similarity {
                results.push(SearchResult {
                    id: id.clone(),
                    similarity,
                    entry: entry.clone(),
                });
            }
        }

        drop(entries);

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results.into_iter().take(top_k).collect())
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut entries = self.entries.write().await;
        entries.remove(id).is_some()
    }

    #[allow(dead_code)]
    pub async fn get(&self, id: &str) -> Option<VectorEntry> {
        let entries = self.entries.read().await;
        entries.get(id).cloned()
    }

    #[allow(dead_code)]
    pub async fn contains(&self, id: &str) -> bool {
        let entries = self.entries.read().await;
        entries.contains_key(id)
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    #[allow(dead_code)]
    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    #[allow(dead_code)]
    pub async fn clear(&self) -> VectorResult<()> {
        let mut entries = self.entries.write().await;
        entries.clear();
        Ok(())
    }

    pub async fn get_stats(&self) -> IndexStats {
        let entries = self.entries.read().await;

        let total_size: usize = entries
            .values()
            .map(|e| {
                e.embedding.len() * 4
                    + e.id.len()
                    + e.metadata.as_ref().map_or(0, |m| m.to_string().len())
            })
            .sum();

        IndexStats {
            total_vectors: entries.len(),
            dimension: self.dimension,
            index_size_bytes: total_size as u64,
            last_updated: chrono::Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub async fn save_to_file(&self, filename: &str) -> VectorResult<PathBuf> {
        let entries = self.entries.read().await;

        let data: HashMap<String, VectorEntry> = entries
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let json_data = serde_json::to_string_pretty(&data)
            .map_err(|e| VectorIndexError::SerializationError(e.to_string()))?;

        drop(entries);

        if !self.index_dir.exists() {
            std::fs::create_dir_all(&self.index_dir)?;
        }

        let file_path = self.index_dir.join(filename);
        std::fs::write(&file_path, json_data)?;

        tracing::info!(
            path = %file_path.display(),
            "向量索引已保存"
        );

        Ok(file_path)
    }

    #[allow(dead_code)]
    pub async fn load_from_file(&self, filename: &str) -> VectorResult<usize> {
        let file_path = self.index_dir.join(filename);

        if !file_path.exists() {
            return Err(VectorIndexError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("文件不存在: {}", file_path.display()),
            )));
        }

        let json_data = std::fs::read_to_string(&file_path)?;

        let loaded_entries: HashMap<String, VectorEntry> = serde_json::from_str(&json_data)
            .map_err(|e| VectorIndexError::DeserializationError(e.to_string()))?;

        let count = loaded_entries.len();

        let mut entries = self.entries.write().await;
        for (id, entry) in loaded_entries.into_iter() {
            entries.insert(id, entry);
        }
        drop(entries);

        tracing::info!(
            path = %file_path.display(),
            count,
            "向量索引已加载"
        );

        Ok(count)
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
