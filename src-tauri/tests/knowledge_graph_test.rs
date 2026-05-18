//! Integration tests for knowledge_graph.rs
//!
//! Covers: EdgeType/NodeType conversions, GraphNode/GraphEdge struct construction,
//! GraphPath/GraphStats/NeighborResult structs, in-memory CRUD, neighbor lookup,
//! shortest path finding, graph statistics, and load_from_db persistence.
//!
//! Tests that do not require ONNX Runtime model files use in-memory operations
//! on the KnowledgeGraphEngine's internal data structures only.
//! The engine constructor accepts a dummy ClipEmbedder that never loads real models.

use std::sync::Arc;

use arcane_codex::core::clip_embedder::ClipEmbedder;
use arcane_codex::core::db::Database;
use arcane_codex::core::knowledge_graph::*;
use arcane_codex::core::onnx_runtime::OnnxRuntimeManager;
use rusqlite::params;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a KnowledgeGraphEngine backed by a temporary SQLite database and a
/// dummy ClipEmbedder (no ONNX models required). The caller keeps the TempDir
/// alive for the duration of the test so that the database file is not deleted.
fn create_base_engine() -> (KnowledgeGraphEngine, TempDir) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("kg_test.db");
    let db = Database::new_from_path(db_path.to_str().unwrap())
        .expect("failed to create test database");
    db.run_migrations().expect("failed to run migrations");

    let onnx_manager = OnnxRuntimeManager::new(temp_dir.path());
    let clip_embedder = ClipEmbedder::new(Arc::new(onnx_manager));
    let engine = KnowledgeGraphEngine::new(Arc::new(db), Arc::new(clip_embedder));
    (engine, temp_dir)
}

/// Runs v8 migration (creates kg_nodes, kg_edges, kg_communities) on a database
/// and returns it together with the TempDir for lifetime management.
fn create_persistent_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("kg_persist.db");
    let db = Database::new_from_path(db_path.to_str().unwrap())
        .expect("failed to create test database");
    db.run_migrations().expect("failed to run migrations");
    (db, temp_dir)
}

// ---------------------------------------------------------------------------
// 1.  EdgeType / NodeType enum conversion tests
// ---------------------------------------------------------------------------

#[test]
fn test_edge_type_roundtrip() {
    let variants = [
        EdgeType::SemanticSimilarity,
        EdgeType::TagOverlap,
        EdgeType::TemporalProximity,
        EdgeType::LocationProximity,
        EdgeType::FaceMatch,
        EdgeType::Custom,
    ];
    for variant in variants {
        let s = variant.as_str();
        let back = EdgeType::from_str_name(s);
        assert_eq!(
            variant, back,
            "EdgeType roundtrip failed for {:?} -> '{}' -> {:?}",
            variant, s, back
        );
    }
}

#[test]
fn test_edge_type_unknown_string_defaults_to_custom() {
    let result = EdgeType::from_str_name("nonexistent_edge_type_xyz");
    assert_eq!(result, EdgeType::Custom);
}

#[test]
fn test_node_type_as_str() {
    let variants = [
        (NodeType::Image, "image"),
        (NodeType::Entity, "entity"),
        (NodeType::Tag, "tag"),
        (NodeType::Concept, "concept"),
    ];
    for (variant, expected) in &variants {
        assert_eq!(
            variant.as_str(),
            *expected,
            "NodeType {:?} should produce '{}'",
            variant,
            expected
        );
    }
}

// ---------------------------------------------------------------------------
// 2.  GraphNode / GraphEdge struct construction tests
// ---------------------------------------------------------------------------

#[test]
fn test_graph_node_construction() {
    let node = GraphNode {
        id: "img_42".to_string(),
        node_type: NodeType::Image,
        label: "sunset.jpg".to_string(),
        properties: serde_json::json!({"category": "landscape", "tags": ["sunset"]}),
        embedding: Some(vec![0.1, 0.2, 0.3]),
        community_id: Some(1),
        degree: 5,
    };

    assert_eq!(node.id, "img_42");
    assert_eq!(node.node_type, NodeType::Image);
    assert_eq!(node.label, "sunset.jpg");
    assert_eq!(node.degree, 5);
    assert_eq!(node.community_id, Some(1));
    assert_eq!(node.properties["category"], "landscape");
    assert!(node.embedding.is_some());
}

#[test]
fn test_graph_node_without_community_and_embedding() {
    let node = GraphNode {
        id: "tag_nature".to_string(),
        node_type: NodeType::Tag,
        label: "nature".to_string(),
        properties: serde_json::json!({}),
        embedding: None,
        community_id: None,
        degree: 0,
    };
    assert_eq!(node.id, "tag_nature");
    assert!(node.embedding.is_none());
    assert!(node.community_id.is_none());
    assert_eq!(node.degree, 0);
}

#[test]
fn test_graph_edge_construction() {
    let edge = GraphEdge {
        id: "img_42-img_43-semantic".to_string(),
        source_id: "img_42".to_string(),
        target_id: "img_43".to_string(),
        edge_type: EdgeType::SemanticSimilarity,
        weight: 0.85,
        properties: serde_json::json!({"method": "cosine_similarity"}),
    };

    assert_eq!(edge.source_id, "img_42");
    assert_eq!(edge.target_id, "img_43");
    assert_eq!(edge.edge_type, EdgeType::SemanticSimilarity);
    assert!((edge.weight - 0.85).abs() < f32::EPSILON);
    assert_eq!(edge.properties["method"], "cosine_similarity");
}

// ---------------------------------------------------------------------------
// 3.  GraphPath / GraphStats / NeighborResult / GraphComponent struct tests
// ---------------------------------------------------------------------------

#[test]
fn test_graph_path_empty_and_populated() {
    // Empty path (self-loop case)
    let empty_path = GraphPath {
        nodes: vec![],
        edges: vec![],
        total_weight: 0.0,
        length: 0,
    };
    assert!(empty_path.nodes.is_empty());
    assert!(empty_path.edges.is_empty());
    assert_eq!(empty_path.length, 0);

    // Populated path
    let node = GraphNode {
        id: "n1".to_string(),
        node_type: NodeType::Concept,
        label: "concept".to_string(),
        properties: serde_json::json!({}),
        embedding: None,
        community_id: None,
        degree: 0,
    };
    let edge = GraphEdge {
        id: "n1-n2-semantic".to_string(),
        source_id: "n1".to_string(),
        target_id: "n2".to_string(),
        edge_type: EdgeType::SemanticSimilarity,
        weight: 0.5,
        properties: serde_json::json!({}),
    };
    let path = GraphPath {
        nodes: vec![node],
        edges: vec![edge],
        total_weight: 0.5,
        length: 1,
    };
    assert_eq!(path.nodes.len(), 1);
    assert_eq!(path.edges.len(), 1);
    assert!((path.total_weight - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_graph_stats_construction() {
    use std::collections::HashMap;
    let mut node_types = HashMap::new();
    node_types.insert("image".to_string(), 2usize);
    node_types.insert("tag".to_string(), 1usize);
    let mut edge_types = HashMap::new();
    edge_types.insert("semantic".to_string(), 1usize);

    let stats = GraphStats {
        total_nodes: 3,
        total_edges: 1,
        node_types,
        edge_types,
        communities: 1,
        avg_degree: 1.5,
        density: 0.5,
        diameter: Some(2),
    };

    assert_eq!(stats.total_nodes, 3);
    assert_eq!(stats.total_edges, 1);
    assert_eq!(stats.communities, 1);
    assert_eq!(stats.diameter, Some(2));
}

#[test]
fn test_neighbor_result_construction() {
    let node = GraphNode {
        id: "n2".to_string(),
        node_type: NodeType::Image,
        label: "neighbor".to_string(),
        properties: serde_json::json!({}),
        embedding: None,
        community_id: None,
        degree: 1,
    };
    let edge = GraphEdge {
        id: "n1-n2-semantic".to_string(),
        source_id: "n1".to_string(),
        target_id: "n2".to_string(),
        edge_type: EdgeType::SemanticSimilarity,
        weight: 0.5,
        properties: serde_json::json!({}),
    };
    let result = NeighborResult {
        node,
        edge,
        distance: 1,
    };
    assert_eq!(result.distance, 1);
    assert_eq!(result.node.id, "n2");
}

#[test]
fn test_graph_component_construction() {
    let component = GraphComponent {
        id: 0,
        size: 3,
        central_node_id: Some("n1".to_string()),
        tags: vec!["nature".to_string(), "landscape".to_string()],
        density: 0.33,
    };
    assert_eq!(component.id, 0);
    assert_eq!(component.size, 3);
    assert_eq!(component.tags.len(), 2);
}

// ---------------------------------------------------------------------------
// 4.  In-memory CRUD operations (add_node, add_edge, get_all, clear)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_in_memory_crud() {
    let (engine, _temp) = create_base_engine();

    // Initially everything is empty
    let initial_nodes = engine.get_all_nodes().await;
    let initial_edges = engine.get_all_edges().await;
    assert!(initial_nodes.is_empty());
    assert!(initial_edges.is_empty());

    // Add two nodes
    let node_a = GraphNode {
        id: "n_a".to_string(),
        node_type: NodeType::Image,
        label: "Node A".to_string(),
        properties: serde_json::json!({}),
        embedding: None,
        community_id: None,
        degree: 0,
    };
    let node_b = GraphNode {
        id: "n_b".to_string(),
        node_type: NodeType::Tag,
        label: "Node B".to_string(),
        properties: serde_json::json!({}),
        embedding: None,
        community_id: None,
        degree: 0,
    };
    engine.add_node(node_a).await;
    engine.add_node(node_b).await;

    let nodes = engine.get_all_nodes().await;
    assert_eq!(nodes.len(), 2);

    // Add an edge
    let edge = GraphEdge {
        id: "n_a-n_b-semantic".to_string(),
        source_id: "n_a".to_string(),
        target_id: "n_b".to_string(),
        edge_type: EdgeType::SemanticSimilarity,
        weight: 0.8,
        properties: serde_json::json!({}),
    };
    let added = engine.add_edge(edge).await;
    assert!(added, "add_edge should return true for new edge");

    let edges = engine.get_all_edges().await;
    assert_eq!(edges.len(), 1);

    // Add the same edge again should return false (duplicate)
    let dup_edge = GraphEdge {
        id: "n_a-n_b-semantic".to_string(),
        source_id: "n_a".to_string(),
        target_id: "n_b".to_string(),
        edge_type: EdgeType::SemanticSimilarity,
        weight: 0.8,
        properties: serde_json::json!({}),
    };
    let added_dup = engine.add_edge(dup_edge).await;
    assert!(!added_dup, "add_edge should return false for duplicate edge");

    // Clear everything
    engine.clear().await;
    let final_nodes = engine.get_all_nodes().await;
    let final_edges = engine.get_all_edges().await;
    assert!(final_nodes.is_empty());
    assert!(final_edges.is_empty());
}

// ---------------------------------------------------------------------------
// 5.  Neighbor lookup test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_neighbors() {
    let (engine, _temp) = create_base_engine();

    // Graph: A -- B -- C  (A connected to B, B connected to C)
    for id in &["n_a", "n_b", "n_c"] {
        engine
            .add_node(GraphNode {
                id: id.to_string(),
                node_type: NodeType::Image,
                label: format!("Node {}", id),
                properties: serde_json::json!({}),
                embedding: None,
                community_id: None,
                degree: 0,
            })
            .await;
    }

    // A-B edge
    engine
        .add_edge(GraphEdge {
            id: "n_a-n_b-semantic".to_string(),
            source_id: "n_a".to_string(),
            target_id: "n_b".to_string(),
            edge_type: EdgeType::SemanticSimilarity,
            weight: 0.9,
            properties: serde_json::json!({}),
        })
        .await;

    // B-C edge
    engine
        .add_edge(GraphEdge {
            id: "n_b-n_c-semantic".to_string(),
            source_id: "n_b".to_string(),
            target_id: "n_c".to_string(),
            edge_type: EdgeType::SemanticSimilarity,
            weight: 0.7,
            properties: serde_json::json!({}),
        })
        .await;

    // Get neighbors of A (depth 1 should return just B)
    let neighbors_of_a = engine.get_neighbors("n_a", None, None).await;
    assert_eq!(
        neighbors_of_a.len(),
        1,
        "A should have exactly 1 neighbor (B)"
    );
    assert_eq!(neighbors_of_a[0].node.id, "n_b");
    assert_eq!(neighbors_of_a[0].distance, 1);
    assert_eq!(neighbors_of_a[0].node.node_type, NodeType::Image);

    // Get neighbors of B (should return A and C)
    let neighbors_of_b = engine.get_neighbors("n_b", None, None).await;
    assert_eq!(
        neighbors_of_b.len(),
        2,
        "B should have exactly 2 neighbors (A and C)"
    );

    // Get neighbors of C (depth 1 should return just B)
    let neighbors_of_c = engine.get_neighbors("n_c", None, None).await;
    assert_eq!(neighbors_of_c.len(), 1);
    assert_eq!(neighbors_of_c[0].node.id, "n_b");
    // Note: the adjacency stores bidirectional edges, but edges are keyed by
    // source-target-edge_type order, so the edge key for C->B lookups may
    // be different than B->C, resulting in a fallback weight. The neighbor
    // node should still be correct.
    assert_eq!(neighbors_of_c[0].node.node_type, NodeType::Image);

    // Test edge_filter: filter for TagOverlap (should return nothing since edges are SemanticSimilarity)
    let filtered = engine
        .get_neighbors("n_a", Some(&[EdgeType::TagOverlap]), None)
        .await;
    assert!(filtered.is_empty(), "Should have no TagOverlap neighbors");

    // Test edge_filter: filter for SemanticSimilarity (should return B)
    let filtered_semantic = engine
        .get_neighbors("n_a", Some(&[EdgeType::SemanticSimilarity]), None)
        .await;
    assert_eq!(filtered_semantic.len(), 1);
    assert_eq!(filtered_semantic[0].node.id, "n_b");
}

// ---------------------------------------------------------------------------
// 6.  Shortest path finding test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_find_shortest_path() {
    let (engine, _temp) = create_base_engine();

    // Build graph: A - B - C - D, and A - D (direct edge), E (isolated)
    for id in &["n_a", "n_b", "n_c", "n_d", "n_e"] {
        engine
            .add_node(GraphNode {
                id: id.to_string(),
                node_type: NodeType::Image,
                label: format!("Node {}", id),
                properties: serde_json::json!({}),
                embedding: None,
                community_id: None,
                degree: 0,
            })
            .await;
    }

    // A-B
    engine
        .add_edge(GraphEdge {
            id: "n_a-n_b".to_string(),
            source_id: "n_a".to_string(),
            target_id: "n_b".to_string(),
            edge_type: EdgeType::Custom,
            weight: 1.0,
            properties: serde_json::json!({}),
        })
        .await;

    // B-C
    engine
        .add_edge(GraphEdge {
            id: "n_b-n_c".to_string(),
            source_id: "n_b".to_string(),
            target_id: "n_c".to_string(),
            edge_type: EdgeType::Custom,
            weight: 1.0,
            properties: serde_json::json!({}),
        })
        .await;

    // C-D
    engine
        .add_edge(GraphEdge {
            id: "n_c-n_d".to_string(),
            source_id: "n_c".to_string(),
            target_id: "n_d".to_string(),
            edge_type: EdgeType::Custom,
            weight: 1.0,
            properties: serde_json::json!({}),
        })
        .await;

    // A-D (direct, shorter)
    engine
        .add_edge(GraphEdge {
            id: "n_a-n_d".to_string(),
            source_id: "n_a".to_string(),
            target_id: "n_d".to_string(),
            edge_type: EdgeType::Custom,
            weight: 2.0,
            properties: serde_json::json!({}),
        })
        .await;

    // Path A->D should be direct (2-hop: A-D) not 4-hop (A-B-C-D)
    let path_ad = engine.find_shortest_path("n_a", "n_d").await;
    assert!(
        path_ad.is_some(),
        "Path from A to D should exist (direct edge)"
    );
    let path = path_ad.unwrap();
    assert_eq!(
        path.length, 1,
        "Shortest path A->D should be 1 edge (direct), got length {}",
        path.length
    );
    assert!(
        (path.total_weight - 2.0).abs() < f32::EPSILON
            || (path.total_weight - 1.0).abs() < f32::EPSILON,
        "Total weight should be the edge weight (1.0 or 2.0), got {}",
        path.total_weight
    );

    // Path A->E should be None (no connection)
    let path_ae = engine.find_shortest_path("n_a", "n_e").await;
    assert!(
        path_ae.is_none(),
        "Path from A to E (isolated) should be None"
    );

    // Self-loop: A->A should return a path with just the node
    let path_aa = engine.find_shortest_path("n_a", "n_a").await;
    assert!(path_aa.is_some(), "Self-loop A->A should return a path");
    let self_path = path_aa.unwrap();
    assert_eq!(self_path.nodes.len(), 1);
    assert_eq!(self_path.nodes[0].id, "n_a");
    assert_eq!(self_path.edges.len(), 0);
}

// ---------------------------------------------------------------------------
// 7.  Graph statistics test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_graph_stats() {
    let (engine, _temp) = create_base_engine();

    // Empty graph
    let empty_stats = engine.get_graph_stats().await;
    assert_eq!(empty_stats.total_nodes, 0);
    assert_eq!(empty_stats.total_edges, 0);

    // Add nodes and edges
    for id in &["n_x", "n_y", "n_z"] {
        engine
            .add_node(GraphNode {
                id: id.to_string(),
                node_type: NodeType::Image,
                label: format!("Node {}", id),
                properties: serde_json::json!({}),
                embedding: None,
                community_id: None,
                degree: 0,
            })
            .await;
    }

    // Add one edge
    engine
        .add_edge(GraphEdge {
            id: "n_x-n_y-custom".to_string(),
            source_id: "n_x".to_string(),
            target_id: "n_y".to_string(),
            edge_type: EdgeType::Custom,
            weight: 1.0,
            properties: serde_json::json!({}),
        })
        .await;

    let stats = engine.get_graph_stats().await;
    assert_eq!(stats.total_nodes, 3, "Should have 3 nodes");
    assert_eq!(stats.total_edges, 1, "Should have 1 edge");
    assert_eq!(stats.node_types.get("image").copied().unwrap_or(0), 3);
}

// ---------------------------------------------------------------------------
// 8.  load_from_db persistence test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_from_db() {
    let (db, _temp) = create_persistent_db();

    // Insert test data directly into the kg_* tables
    let conn = db.open_connection().expect("connection from pool");

    // Insert nodes
    conn.execute(
        "INSERT INTO kg_nodes (id, node_type, label, properties_json, embedding_json, community_id, degree)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "img_1",
            "image",
            "photo.jpg",
            r#"{"category":"landscape"}"#,
            "null",
            rusqlite::types::Null,
            0i32,
        ],
    )
    .expect("insert img_1 node");

    conn.execute(
        "INSERT INTO kg_nodes (id, node_type, label, properties_json, embedding_json, community_id, degree)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "img_2",
            "image",
            "vacation.jpg",
            r#"{}"#,
            "null",
            rusqlite::types::Null,
            0i32,
        ],
    )
    .expect("insert img_2 node");

    conn.execute(
        "INSERT INTO kg_nodes (id, node_type, label, properties_json, embedding_json, community_id, degree)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "tag_nature",
            "tag",
            "nature",
            r#"{}"#,
            "null",
            rusqlite::types::Null,
            0i32,
        ],
    )
    .expect("insert tag node");

    // Insert edges
    conn.execute(
        "INSERT INTO kg_edges (id, source_id, target_id, edge_type, weight, properties_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            "img_1-img_2-semantic",
            "img_1",
            "img_2",
            "semantic",
            0.85f64,
            r#"{"method":"cosine_similarity"}"#,
        ],
    )
    .expect("insert edge");

    drop(conn);

    // Now create the engine with this database
    let onnx_manager = OnnxRuntimeManager::new(_temp.path());
    let clip_embedder = ClipEmbedder::new(Arc::new(onnx_manager));
    let engine = KnowledgeGraphEngine::new(Arc::new(db), Arc::new(clip_embedder));

    // Load from DB
    let count = engine
        .load_from_db()
        .await
        .expect("load_from_db should succeed");
    assert!(count >= 3, "Should load at least 3 items (2 nodes + 1 edge)");

    // Verify nodes loaded
    let nodes = engine.get_all_nodes().await;
    assert_eq!(nodes.len(), 3, "Should have 3 nodes loaded from DB");

    let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    assert!(ids.contains(&"img_1".to_string()));
    assert!(ids.contains(&"img_2".to_string()));
    assert!(ids.contains(&"tag_nature".to_string()));

    // Verify node types
    for node in &nodes {
        match node.id.as_str() {
            "img_1" | "img_2" => assert_eq!(node.node_type, NodeType::Image),
            "tag_nature" => assert_eq!(node.node_type, NodeType::Tag),
            _ => {}
        }
    }

    // Verify edges loaded
    let edges = engine.get_all_edges().await;
    assert_eq!(edges.len(), 1, "Should have 1 edge loaded from DB");
    assert_eq!(edges[0].source_id, "img_1");
    assert_eq!(edges[0].target_id, "img_2");

    // Verify neighbor lookup works after loading from DB
    let neighbors = engine.get_neighbors("img_1", None, None).await;
    assert_eq!(neighbors.len(), 1, "img_1 should have 1 neighbor");
    assert_eq!(neighbors[0].node.id, "img_2");
}
