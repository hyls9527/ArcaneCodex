#![allow(missing_docs)]
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::clip_embedder::ClipEmbedder;
use crate::core::db::Database;
use crate::core::vector_index::{cosine_similarity, BruteForceVectorIndex};

type AdjacencyMap = Arc<tokio::sync::RwLock<HashMap<String, HashSet<(String, String)>>>>;
type NodeMap = Arc<tokio::sync::RwLock<HashMap<String, GraphNode>>>;
type EdgeMap = Arc<tokio::sync::RwLock<HashMap<String, GraphEdge>>>;
type CommunityList = Arc<tokio::sync::RwLock<Vec<GraphComponent>>>;
type ImageDataRow = (
    i64,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    SemanticSimilarity,
    TagOverlap,
    TemporalProximity,
    LocationProximity,
    FaceMatch,
    Custom,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::SemanticSimilarity => "semantic",
            EdgeType::TagOverlap => "tag_overlap",
            EdgeType::TemporalProximity => "temporal",
            EdgeType::LocationProximity => "location",
            EdgeType::FaceMatch => "face_match",
            EdgeType::Custom => "custom",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s {
            "semantic" => EdgeType::SemanticSimilarity,
            "tag_overlap" => EdgeType::TagOverlap,
            "temporal" => EdgeType::TemporalProximity,
            "location" => EdgeType::LocationProximity,
            "face_match" => EdgeType::FaceMatch,
            _ => EdgeType::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub community_id: Option<u32>,
    pub degree: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Image,
    Entity,
    Tag,
    Concept,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Image => "image",
            NodeType::Entity => "entity",
            NodeType::Tag => "tag",
            NodeType::Concept => "concept",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphComponent {
    pub id: u32,
    pub size: usize,
    pub central_node_id: Option<String>,
    pub tags: Vec<String>,
    pub density: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub total_weight: f32,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_types: HashMap<String, usize>,
    pub edge_types: HashMap<String, usize>,
    pub communities: usize,
    pub avg_degree: f32,
    pub density: f32,
    pub diameter: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborResult {
    pub node: GraphNode,
    pub edge: GraphEdge,
    pub distance: usize,
}

pub struct KnowledgeGraphEngine {
    db: Arc<Database>,
    clip_embedder: Arc<ClipEmbedder>,
    vector_index: Arc<BruteForceVectorIndex>,
    nodes: NodeMap,
    edges: EdgeMap,
    adjacency: AdjacencyMap,
    communities: CommunityList,
    edge_thresholds: HashMap<EdgeType, f32>,
}

impl KnowledgeGraphEngine {
    pub fn new(
        db: Arc<Database>,
        clip_embedder: Arc<ClipEmbedder>,
        vector_index: Arc<BruteForceVectorIndex>,
    ) -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(EdgeType::SemanticSimilarity, 0.7);
        thresholds.insert(EdgeType::TagOverlap, 0.3);
        thresholds.insert(EdgeType::TemporalProximity, 86400.0);
        thresholds.insert(EdgeType::FaceMatch, 0.6);

        tracing::info!("初始化知识图谱引擎");
        Self {
            db,
            clip_embedder,
            vector_index,
            nodes: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            edges: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            adjacency: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            communities: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            edge_thresholds: thresholds,
        }
    }

    pub async fn build_from_images(&self) -> Result<usize, String> {
        tracing::info!("开始构建知识图谱");

        let conn = self.db.open_connection().map_err(|e| e.to_string())?;

        let image_data: Vec<ImageDataRow> = conn
            .prepare("SELECT id, file_path, ai_tags, ai_description, ai_category, ai_status FROM images WHERE ai_status IN ('verified', 'provisional', 'completed')")
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        drop(conn);

        let mut added = 0usize;

        for (image_id, file_path, tags_json, description, category, ai_status) in image_data {
            let node_id = format!("img_{}", image_id);

            let tags: Vec<String> = tags_json
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default();

            let mut props = serde_json::Map::new();
            props.insert("image_id".into(), serde_json::json!(image_id));
            props.insert("file_path".into(), serde_json::json!(file_path));
            if let Some(desc) = description.clone() {
                props.insert("description".into(), serde_json::json!(desc));
            }
            if let Some(cat) = category {
                props.insert("category".into(), serde_json::json!(cat));
            }
            if let Some(status) = ai_status {
                props.insert("ai_status".into(), serde_json::json!(status));
            }
            if !tags.is_empty() {
                props.insert("tags".into(), serde_json::json!(tags.clone()));
            }

            let embedding = self
                .generate_node_embedding(&file_path, &description, &tags)
                .await;

            let node = GraphNode {
                id: node_id.clone(),
                node_type: NodeType::Image,
                label: file_path
                    .rsplit('\\')
                    .next()
                    .or_else(|| file_path.rsplit('/').next())
                    .unwrap_or(&file_path)
                    .to_string(),
                properties: serde_json::Value::Object(props),
                embedding,
                community_id: None,
                degree: 0,
            };

            self.add_node(node).await;
            added += 1;
        }

        tracing::info!("已添加 {} 个图像节点到知识图谱", added);

        self.build_tag_nodes().await?;
        self.discover_semantic_edges(EdgeType::SemanticSimilarity, 0.7)
            .await?;
        self.discover_tag_edges().await?;
        self.find_connected_components().await?;
        self.update_degrees().await;

        if let Err(e) = self.save_to_db().await {
            tracing::warn!("保存知识图谱到数据库失败: {}", e);
        }

        Ok(added)
    }

    async fn generate_node_embedding(
        &self,
        file_path: &str,
        description: &Option<String>,
        tags: &[String],
    ) -> Option<Vec<f32>> {
        let path = std::path::Path::new(file_path);
        if path.exists() {
            if let Ok(clip_result) = self.clip_embedder.embed_image(path).await {
                return Some(clip_result.embedding);
            }
        }

        let mut text_parts: Vec<String> = Vec::new();
        if let Some(desc) = description {
            text_parts.push(desc.clone());
        }
        for tag in tags {
            text_parts.push(tag.clone());
        }

        if text_parts.is_empty() {
            return None;
        }

        let combined_text = text_parts.join(" ");
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        combined_text.hash(&mut hasher);
        let hash_val = hasher.finish();

        let mut embedding = Vec::with_capacity(512);
        for i in 0..512 {
            let val = ((hash_val.wrapping_add(i as u64 * 2654435761u64)) as f32) / u64::MAX as f32;
            embedding.push(val * 2.0 - 1.0);
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        Some(embedding)
    }

    pub async fn add_node(&self, node: GraphNode) {
        let node_id = node.id.clone();
        let mut nodes = self.nodes.write().await;
        let mut adjacency = self.adjacency.write().await;
        nodes.insert(node_id.clone(), node);
        adjacency.entry(node_id).or_insert_with(HashSet::new);
    }

    pub async fn add_edge(&self, edge: GraphEdge) -> bool {
        let edge_key = format!(
            "{}-{}-{}",
            edge.source_id,
            edge.target_id,
            edge.edge_type.as_str()
        );

        let exists = {
            let edges = self.edges.read().await;
            edges.contains_key(&edge_key)
        };

        if exists {
            return false;
        }

        {
            let mut edges = self.edges.write().await;
            edges.insert(edge_key.clone(), edge.clone());
        }

        {
            let mut adjacency = self.adjacency.write().await;
            adjacency
                .entry(edge.source_id.clone())
                .or_insert_with(HashSet::new)
                .insert((edge.target_id.clone(), edge.edge_type.as_str().to_string()));
            adjacency
                .entry(edge.target_id.clone())
                .or_insert_with(HashSet::new)
                .insert((edge.source_id.clone(), edge.edge_type.as_str().to_string()));
        }

        true
    }

    pub async fn discover_semantic_edges(
        &self,
        edge_type: EdgeType,
        threshold: f32,
    ) -> Result<usize, String> {
        let nodes = self.nodes.read().await;
        let node_ids: Vec<String> = nodes.keys().cloned().collect();
        drop(nodes);

        let mut added = 0usize;
        let chunk_size = 50;

        for chunk in node_ids.chunks(chunk_size) {
            for (i, id_a) in chunk.iter().enumerate() {
                for id_b in chunk.iter().skip(i + 1) {
                    let emb_a = self.get_node_embedding(id_a).await;
                    let emb_b = self.get_node_embedding(id_b).await;

                    if let (Some(a), Some(b)) = (emb_a, emb_b) {
                        let similarity = cosine_similarity(&a, &b);
                        if similarity >= threshold {
                            let edge = GraphEdge {
                                id: format!("{}-{}-{}", id_a, id_b, edge_type.as_str()),
                                source_id: id_a.clone(),
                                target_id: id_b.clone(),
                                edge_type,
                                weight: similarity,
                                properties: serde_json::json!({
                                    "method": "cosine_similarity"
                                }),
                            };

                            if self.add_edge(edge).await {
                                added += 1;
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            edge_type = edge_type.as_str(),
            count = added,
            threshold,
            "语义边发现完成"
        );

        Ok(added)
    }

    async fn get_node_embedding(&self, node_id: &str) -> Option<Vec<f32>> {
        let nodes = self.nodes.read().await;
        if let Some(node) = nodes.get(node_id) {
            if let Some(ref emb) = node.embedding {
                return Some(emb.clone());
            }
        }
        None
    }

    async fn build_tag_nodes(&self) -> Result<(), String> {
        let nodes = self.nodes.read().await;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for node in nodes.values() {
            if let Some(tags) = node.properties.get("tags") {
                if let Some(arr) = tags.as_array() {
                    for tag in arr {
                        if let Some(s) = tag.as_str() {
                            *tag_counts.entry(s.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        drop(nodes);

        for (tag, count) in tag_counts.iter().filter(|(_, c)| **c >= 2) {
            let node = GraphNode {
                id: format!("tag_{}", tag),
                node_type: NodeType::Tag,
                label: tag.clone(),
                properties: serde_json::json!({
                    "count": count,
                    "tag_type": "auto"
                }),
                embedding: None,
                community_id: None,
                degree: 0,
            };
            self.add_node(node).await;
        }

        Ok(())
    }

    async fn discover_tag_edges(&self) -> Result<usize, String> {
        let nodes = self.nodes.read().await;
        let mut added = 0usize;

        let image_nodes: Vec<GraphNode> = nodes
            .values()
            .filter(|n| n.node_type == NodeType::Image)
            .cloned()
            .collect();

        for img_node in &image_nodes {
            if let Some(tags) = img_node.properties.get("tags") {
                if let Some(arr) = tags.as_array() {
                    for tag in arr {
                        if let Some(tag_str) = tag.as_str() {
                            let tag_node_id = format!("tag_{}", tag_str);
                            if nodes.contains_key(&tag_node_id) {
                                let edge = GraphEdge {
                                    id: format!("{}-{}-tag", img_node.id, tag_node_id),
                                    source_id: img_node.id.clone(),
                                    target_id: tag_node_id,
                                    edge_type: EdgeType::TagOverlap,
                                    weight: 1.0,
                                    properties: serde_json::json!({}),
                                };

                                if self.add_edge(edge).await {
                                    added += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        drop(nodes);
        tracing::info!(count = added, "标签边发现完成");
        Ok(added)
    }

    pub async fn find_connected_components(&self) -> Result<u32, String> {
        let adjacency = self.adjacency.read().await;
        let nodes = self.nodes.read().await;

        let mut visited: HashSet<String> = HashSet::new();
        let mut communities_vec: Vec<HashSet<String>> = Vec::new();

        for node_id in nodes.keys() {
            if visited.contains(node_id) {
                continue;
            }

            let mut component = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(node_id.clone());

            while let Some(current) = queue.pop_front() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());
                component.insert(current.clone());

                if let Some(neighbors) = adjacency.get(&current) {
                    for (neighbor_id, _) in neighbors.iter() {
                        if !visited.contains(neighbor_id) {
                            queue.push_back(neighbor_id.clone());
                        }
                    }
                }
            }

            if !component.is_empty() {
                communities_vec.push(component);
            }
        }

        drop(adjacency);
        drop(nodes);

        let mut communities_output = Vec::new();

        for (community_id, component) in communities_vec.into_iter().enumerate() {
            let size = component.len();
            let mut tags: HashSet<String> = HashSet::new();

            let nodes_ref = self.nodes.read().await;
            let mut max_degree = 0u32;
            let mut central_node = None;

            for node_id in &component {
                if let Some(node) = nodes_ref.get(node_id) {
                    if node.degree > max_degree {
                        max_degree = node.degree;
                        central_node = Some(node.id.clone());
                    }

                    if node.node_type == NodeType::Tag {
                        tags.insert(node.label.clone());
                    } else if let Some(t) = node.properties.get("tags") {
                        if let Some(arr) = t.as_array() {
                            for tag in arr {
                                if let Some(s) = tag.as_str() {
                                    tags.insert(s.to_string());
                                }
                            }
                        }
                    }
                }
            }

            let internal_edges = self.count_internal_edges(&component).await;
            let possible_edges = size * (size - 1) / 2;
            let density = if possible_edges > 0 {
                internal_edges as f32 / possible_edges as f32
            } else {
                0.0
            };

            let community_id_u32 = community_id as u32;

            for node_id in &component {
                let mut nodes_mut = self.nodes.write().await;
                if let Some(node) = nodes_mut.get_mut(node_id) {
                    node.community_id = Some(community_id_u32);
                }
            }

            communities_output.push(GraphComponent {
                id: community_id_u32,
                size,
                central_node_id: central_node,
                tags: tags.into_iter().collect(),
                density,
            });
        }

        let count = communities_output.len() as u32;

        {
            let mut comm = self.communities.write().await;
            *comm = communities_output;
        }

        tracing::info!(communities = count, "社区检测完成");

        Ok(count)
    }

    async fn count_internal_edges(&self, component: &HashSet<String>) -> usize {
        let edges = self.edges.read().await;
        let mut count = 0usize;

        for edge in edges.values() {
            if component.contains(&edge.source_id) && component.contains(&edge.target_id) {
                count += 1;
            }
        }

        count
    }

    async fn update_degrees(&self) {
        let adjacency = self.adjacency.read().await;
        let mut nodes = self.nodes.write().await;

        for (node_id, neighbors) in adjacency.iter() {
            if let Some(node) = nodes.get_mut(node_id) {
                node.degree = neighbors.len() as u32;
            }
        }
    }

    pub async fn get_neighbors(
        &self,
        node_id: &str,
        edge_filter: Option<&[EdgeType]>,
        limit: Option<usize>,
    ) -> Vec<NeighborResult> {
        let adjacency = self.adjacency.read().await;
        let edges = self.edges.read().await;
        let nodes = self.nodes.read().await;

        let mut results = Vec::new();

        if let Some(neighbors) = adjacency.get(node_id) {
            for (neighbor_id, edge_type_str) in neighbors.iter() {
                if let Some(filter) = edge_filter {
                    let et = EdgeType::from_str_name(edge_type_str);
                    if !filter.contains(&et) {
                        continue;
                    }
                }

                if let Some(node) = nodes.get(neighbor_id) {
                    let edge_id = format!("{}-{}-{}", node_id, neighbor_id, edge_type_str);
                    let edge = edges.get(&edge_id).cloned().unwrap_or(GraphEdge {
                        id: edge_id.clone(),
                        source_id: node_id.to_string(),
                        target_id: neighbor_id.clone(),
                        edge_type: EdgeType::from_str_name(edge_type_str),
                        weight: 0.0,
                        properties: serde_json::json!({}),
                    });

                    results.push(NeighborResult {
                        node: node.clone(),
                        edge,
                        distance: 1,
                    });
                }
            }
        }

        drop(nodes);
        drop(edges);
        drop(adjacency);

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results.sort_by(|a, b| {
            b.edge
                .weight
                .partial_cmp(&a.edge.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    pub async fn find_shortest_path(&self, source_id: &str, target_id: &str) -> Option<GraphPath> {
        let adjacency = self.adjacency.read().await;
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        if source_id == target_id {
            if let Some(node) = nodes.get(source_id) {
                return Some(GraphPath {
                    nodes: vec![node.clone()],
                    edges: vec![],
                    total_weight: 0.0,
                    length: 0,
                });
            }
            return None;
        }

        let mut visited: HashMap<String, (String, String)> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(source_id.to_string());
        visited.insert(source_id.to_string(), (String::new(), String::new()));

        while let Some(current) = queue.pop_front() {
            if current == *target_id {
                break;
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for (neighbor_id, edge_type) in neighbors.iter() {
                    if !visited.contains_key(neighbor_id) {
                        visited.insert(neighbor_id.clone(), (current.clone(), edge_type.clone()));
                        queue.push_back(neighbor_id.clone());
                    }
                }
            }
        }

        if !visited.contains_key(target_id) {
            return None;
        }

        let mut path_nodes = Vec::new();
        let mut path_edges = Vec::new();
        let mut current = target_id.to_string();
        let mut total_weight = 0.0f32;

        loop {
            if let Some(node) = nodes.get(&current) {
                path_nodes.push(node.clone());
            }

            if current == *source_id {
                break;
            }

            let (prev, edge_type) = visited.get(&current)?.clone();
            let edge_id = format!("{}-{}-{}", prev, current, edge_type);

            if let Some(edge) = edges.get(&edge_id) {
                total_weight += edge.weight;
                path_edges.push(edge.clone());
            }

            current = prev;
        }

        path_nodes.reverse();
        path_edges.reverse();

        let path_length = path_edges.len();

        Some(GraphPath {
            nodes: path_nodes,
            edges: path_edges,
            total_weight,
            length: path_length,
        })
    }

    pub async fn get_graph_stats(&self) -> GraphStats {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        let communities = self.communities.read().await;

        let mut node_types: HashMap<String, usize> = HashMap::new();
        let mut edge_types: HashMap<String, usize> = HashMap::new();
        let mut total_degree = 0u64;

        for node in nodes.values() {
            let key = node.node_type.as_str().to_string();
            *node_types.entry(key).or_insert(0) += 1;
            total_degree += node.degree as u64;
        }

        for edge in edges.values() {
            let key = edge.edge_type.as_str().to_string();
            *edge_types.entry(key).or_insert(0) += 1;
        }

        let n = nodes.len();
        let avg_degree = if n > 0 {
            total_degree as f32 / n as f32
        } else {
            0.0
        };

        let possible_edges = if n >= 2 { n * (n - 1) / 2 } else { 0 };
        let density = if possible_edges > 0 {
            edges.len() as f32 / possible_edges as f32
        } else {
            0.0
        };

        GraphStats {
            total_nodes: n,
            total_edges: edges.len(),
            node_types,
            edge_types,
            communities: communities.len(),
            avg_degree,
            density,
            diameter: None,
        }
    }

    pub async fn get_all_nodes(&self) -> Vec<GraphNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    pub async fn get_all_edges(&self) -> Vec<GraphEdge> {
        let edges = self.edges.read().await;
        edges.values().cloned().collect()
    }

    pub async fn get_communities(&self) -> Vec<GraphComponent> {
        let communities = self.communities.read().await;
        communities.clone()
    }

    pub async fn get_community_nodes(&self, community_id: u32) -> Vec<GraphNode> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| n.community_id == Some(community_id))
            .cloned()
            .collect()
    }

    pub async fn search_nodes(&self, query: &str, limit: usize) -> Vec<GraphNode> {
        let nodes = self.nodes.read().await;
        let query_lower = query.to_lowercase();

        nodes
            .values()
            .filter(|n| {
                n.label.to_lowercase().contains(&query_lower)
                    || n.properties
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || n.properties
                        .get("category")
                        .and_then(|c| c.as_str())
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut nodes = self.nodes.write().await;
        let mut edges = self.edges.write().await;
        let mut adjacency = self.adjacency.write().await;
        let mut communities = self.communities.write().await;

        nodes.clear();
        edges.clear();
        adjacency.clear();
        communities.clear();

        tracing::info!("知识图谱已清空");
    }

    pub async fn save_to_db(&self) -> Result<usize, String> {
        let conn = self.db.open_connection().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM kg_edges", [])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM kg_nodes", [])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM kg_communities", [])
            .map_err(|e| e.to_string())?;

        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        let communities = self.communities.read().await;

        let mut stmt = conn
            .prepare(
                "INSERT INTO kg_nodes (id, node_type, label, properties_json, embedding_json, community_id, degree)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(|e| e.to_string())?;

        for node in nodes.values() {
            let props_json = serde_json::to_string(&node.properties).unwrap_or_default();
            let emb_json = node
                .embedding
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default());
            stmt.execute([
                &node.id,
                node.node_type.as_str(),
                &node.label,
                &props_json,
                emb_json.as_deref().unwrap_or("null"),
                &node.community_id.map(|c| c.to_string()).unwrap_or_default(),
                &node.degree.to_string(),
            ])
            .map_err(|e| e.to_string())?;
        }
        drop(stmt);

        let mut stmt = conn
            .prepare(
                "INSERT INTO kg_edges (id, source_id, target_id, edge_type, weight, properties_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| e.to_string())?;

        for edge in edges.values() {
            let props_json = serde_json::to_string(&edge.properties).unwrap_or_default();
            stmt.execute([
                &edge.id,
                &edge.source_id,
                &edge.target_id,
                edge.edge_type.as_str(),
                &edge.weight.to_string(),
                &props_json,
            ])
            .map_err(|e| e.to_string())?;
        }
        drop(stmt);

        let mut stmt = conn
            .prepare(
                "INSERT INTO kg_communities (id, size, central_node_id, tags_json, density)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|e| e.to_string())?;

        for comm in communities.iter() {
            let tags_json = serde_json::to_string(&comm.tags).unwrap_or_default();
            let central = comm.central_node_id.clone().unwrap_or_default();
            stmt.execute([
                &comm.id.to_string(),
                &comm.size.to_string(),
                &central,
                &tags_json,
                &comm.density.to_string(),
            ])
            .map_err(|e| e.to_string())?;
        }
        drop(stmt);

        let total = nodes.len() + edges.len() + communities.len();
        tracing::info!(
            "知识图谱已保存到数据库: {} 节点, {} 边, {} 社区",
            nodes.len(),
            edges.len(),
            communities.len()
        );

        Ok(total)
    }

    pub async fn load_from_db(&self) -> Result<usize, String> {
        let conn = self.db.open_connection().map_err(|e| e.to_string())?;

        let mut nodes = self.nodes.write().await;
        let mut edges = self.edges.write().await;
        let mut adjacency = self.adjacency.write().await;
        let mut communities = self.communities.write().await;

        nodes.clear();
        edges.clear();
        adjacency.clear();
        communities.clear();

        let mut stmt = conn
            .prepare("SELECT id, node_type, label, properties_json, embedding_json, community_id, degree FROM kg_nodes")
            .map_err(|e| e.to_string())?;

        let node_rows = stmt
            .query_map([], |row| {
                let node_type_str: String = row.get(1)?;
                let node_type = match node_type_str.as_str() {
                    "image" => NodeType::Image,
                    "entity" => NodeType::Entity,
                    "tag" => NodeType::Tag,
                    _ => NodeType::Concept,
                };
                let props_json: String = row.get(3)?;
                let properties: serde_json::Value =
                    serde_json::from_str(&props_json).unwrap_or(serde_json::json!({}));
                let emb_json: String = row.get(4)?;
                let embedding: Option<Vec<f32>> = serde_json::from_str(&emb_json).ok();
                let community_id: Option<i64> = row.get(5)?;
                let degree: i64 = row.get(6)?;

                Ok(GraphNode {
                    id: row.get(0)?,
                    node_type,
                    label: row.get(2)?,
                    properties,
                    embedding,
                    community_id: community_id.map(|c| c as u32),
                    degree: degree as u32,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut node_count = 0usize;
        for node in node_rows.filter_map(|r| r.ok()) {
            let node_id = node.id.clone();
            nodes.insert(node_id.clone(), node);
            adjacency.entry(node_id).or_insert_with(HashSet::new);
            node_count += 1;
        }
        drop(stmt);

        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, edge_type, weight, properties_json FROM kg_edges",
            )
            .map_err(|e| e.to_string())?;

        let edge_rows = stmt
            .query_map([], |row| {
                let edge_type_str: String = row.get(3)?;
                let edge_type = EdgeType::from_str_name(&edge_type_str);
                let props_json: String = row.get(5)?;
                let properties: serde_json::Value =
                    serde_json::from_str(&props_json).unwrap_or(serde_json::json!({}));
                let weight: f64 = row.get(4)?;

                Ok(GraphEdge {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    edge_type,
                    weight: weight as f32,
                    properties,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut edge_count = 0usize;
        for edge in edge_rows.filter_map(|r| r.ok()) {
            let edge_key = format!(
                "{}-{}-{}",
                edge.source_id,
                edge.target_id,
                edge.edge_type.as_str()
            );
            adjacency
                .entry(edge.source_id.clone())
                .or_insert_with(HashSet::new)
                .insert((edge.target_id.clone(), edge.edge_type.as_str().to_string()));
            adjacency
                .entry(edge.target_id.clone())
                .or_insert_with(HashSet::new)
                .insert((edge.source_id.clone(), edge.edge_type.as_str().to_string()));
            edges.insert(edge_key, edge);
            edge_count += 1;
        }
        drop(stmt);

        let mut stmt = conn
            .prepare("SELECT id, size, central_node_id, tags_json, density FROM kg_communities")
            .map_err(|e| e.to_string())?;

        let comm_rows = stmt
            .query_map([], |row| {
                let tags_json: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                let central: Option<String> = row.get(2)?;
                let density: f64 = row.get(4)?;

                Ok(GraphComponent {
                    id: row.get::<_, i64>(0)? as u32,
                    size: row.get::<_, i64>(1)? as usize,
                    central_node_id: central.filter(|s| !s.is_empty()),
                    tags,
                    density: density as f32,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut comm_count = 0usize;
        for comm in comm_rows.filter_map(|r| r.ok()) {
            communities.push(comm);
            comm_count += 1;
        }

        tracing::info!(
            "从数据库加载知识图谱: {} 节点, {} 边, {} 社区",
            node_count,
            edge_count,
            comm_count
        );

        Ok(node_count + edge_count + comm_count)
    }

}
