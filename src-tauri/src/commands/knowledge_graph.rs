#![allow(missing_docs)]
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::core::knowledge_graph::{
    EdgeType, GraphComponent, GraphEdge, GraphNode, GraphPath, GraphStats, KnowledgeGraphEngine,
    NeighborResult,
};

pub struct KgState {
    pub engine: Arc<KnowledgeGraphEngine>,
}

#[derive(Debug, Serialize)]
pub struct BuildResult {
    pub success: bool,
    pub nodes_added: usize,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn kg_build_graph(state: State<'_, KgState>) -> Result<BuildResult, String> {
    match state.engine.build_from_images().await {
        Ok(count) => Ok(BuildResult {
            success: true,
            nodes_added: count,
            error: None,
        }),
        Err(e) => Ok(BuildResult {
            success: false,
            nodes_added: 0,
            error: Some(e),
        }),
    }
}

#[tauri::command]
pub async fn kg_get_stats(state: State<'_, KgState>) -> Result<GraphStats, String> {
    Ok(state.engine.get_graph_stats().await)
}

#[tauri::command]
pub async fn kg_get_all_nodes(state: State<'_, KgState>) -> Result<Vec<GraphNode>, String> {
    Ok(state.engine.get_all_nodes().await)
}

#[tauri::command]
pub async fn kg_get_all_edges(state: State<'_, KgState>) -> Result<Vec<GraphEdge>, String> {
    Ok(state.engine.get_all_edges().await)
}

#[tauri::command]
pub async fn kg_get_communities(state: State<'_, KgState>) -> Result<Vec<GraphComponent>, String> {
    Ok(state.engine.get_communities().await)
}

#[tauri::command]
pub async fn kg_get_community_nodes(
    state: State<'_, KgState>,
    community_id: u32,
) -> Result<Vec<GraphNode>, String> {
    Ok(state.engine.get_community_nodes(community_id).await)
}

#[tauri::command]
pub async fn kg_get_neighbors(
    state: State<'_, KgState>,
    node_id: String,
    edge_types: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<Vec<NeighborResult>, String> {
    let filter = edge_types.map(|types| {
        types
            .iter()
            .map(|t| EdgeType::from_str_name(t))
            .collect::<Vec<EdgeType>>()
    });

    let filter_ref = filter.as_deref();

    Ok(state
        .engine
        .get_neighbors(&node_id, filter_ref, limit)
        .await)
}

#[tauri::command]
pub async fn kg_find_path(
    state: State<'_, KgState>,
    source_id: String,
    target_id: String,
) -> Result<Option<GraphPath>, String> {
    Ok(state
        .engine
        .find_shortest_path(&source_id, &target_id)
        .await)
}

#[tauri::command]
pub async fn kg_search_nodes(
    state: State<'_, KgState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<GraphNode>, String> {
    Ok(state.engine.search_nodes(&query, limit.unwrap_or(20)).await)
}

#[tauri::command]
pub async fn kg_clear(state: State<'_, KgState>) -> Result<(), String> {
    state.engine.clear().await;
    Ok(())
}

#[tauri::command]
pub async fn kg_load_from_db(state: State<'_, KgState>) -> Result<BuildResult, String> {
    match state.engine.load_from_db().await {
        Ok(count) => Ok(BuildResult {
            success: true,
            nodes_added: count,
            error: None,
        }),
        Err(e) => Ok(BuildResult {
            success: false,
            nodes_added: 0,
            error: Some(e),
        }),
    }
}

#[tauri::command]
pub async fn kg_save_to_db(state: State<'_, KgState>) -> Result<BuildResult, String> {
    match state.engine.save_to_db().await {
        Ok(count) => Ok(BuildResult {
            success: true,
            nodes_added: count,
            error: None,
        }),
        Err(e) => Ok(BuildResult {
            success: false,
            nodes_added: 0,
            error: Some(e),
        }),
    }
}
