#![allow(missing_docs)]
use std::collections::HashMap;
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
    DimensionMismatch {
        expected: usize,
        got: usize,
    },
    EmptyVector,
    Io(std::io::Error),
}

impl std::fmt::Display for VectorIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorIndexError::DimensionMismatch { expected, got } => {
                write!(f, "ç»´åº¦ä¸å¹é? é¢æ {}, å®é {}", expected, got)
            }
            VectorIndexError::EmptyVector => write!(f, "åéä¸è½ä¸ºç©º"),
            VectorIndexError::Io(e) => write!(f, "IOéè¯¯: {}", e),
        }
    }
}

impl From<std::io::Error> for VectorIndexError {
    fn from(e: std::io::Error) -> Self {
        VectorIndexError::Io(e)
    }
}

pub type VectorResult<T> = std::result::Result<T, VectorIndexError>;

/// Brute-force vector index using HashMap + O(n) linear scan.
/// For large datasets (>10k vectors), replace with HNSW for logarithmic search.
pub struct BruteForceVectorIndex {
    dimension: usize,
    entries: Arc<tokio::sync::RwLock<HashMap<String, VectorEntry>>>,
}

impl BruteForceVectorIndex {
    pub fn new(dimension: usize) -> Self {
        tracing::info!(
            dimension = dimension,
            "åå§åæ´ååéç´¢å¼"
        );
        Self {
            dimension,
            entries: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
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
            let similarity = cosine_similarity(query, &entry.embedding);

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
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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

