#![allow(missing_docs)]
use crate::core::db::Database;
use crate::core::search_index::{SearchFilters, SearchIndexBuilder, SearchResult};
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub page: u32,
    pub page_size: u32,
}

#[tauri::command]
pub async fn semantic_search(
    db: State<'_, Database>,
    request: SearchRequest,
) -> AppResult<SearchResponse> {
    if request.query.trim().is_empty() {
        return Err(AppError::validation("搜索查询不能为空".to_string()));
    }

    let builder = SearchIndexBuilder::new();

    let filters = SearchFilters {
        category: request.category,
        tags: request.tags,
        start_date: request.start_date,
        end_date: request.end_date,
    };

    let page = request.page.unwrap_or(0);
    let page_size = request.page_size.unwrap_or(20);
    let offset = (page * page_size) as usize;

    let mut results = builder.search(
        &db,
        &request.query,
        Some(&filters),
        page_size as usize,
        offset,
    )?;

    if (results.len() as u32) < page_size {
        let remaining = page_size as usize - results.len();
        let existing_ids: Vec<i64> = results.iter().map(|r| r.image_id).collect();

        let conn = db.open_connection().map_err(AppError::database)?;
        let escaped_query = request
            .query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let like_pattern = format!("%{}%", escaped_query);

        let sql = if existing_ids.is_empty() {
            "SELECT DISTINCT n.image_id,
                    i.file_path, i.file_name, i.thumbnail_path,
                    i.ai_description, i.ai_tags, i.ai_category, i.ai_confidence
             FROM narratives n
             JOIN images i ON n.image_id = i.id
             WHERE (n.content LIKE ?1 ESCAPE '\\' OR n.entities_json LIKE ?1 ESCAPE '\\')
             ORDER BY n.created_at DESC
             LIMIT ?2"
                .to_string()
        } else {
            let placeholders: Vec<String> = existing_ids.iter().map(|_| "?".to_string()).collect();
            format!(
                "SELECT DISTINCT n.image_id,
                        i.file_path, i.file_name, i.thumbnail_path,
                        i.ai_description, i.ai_tags, i.ai_category, i.ai_confidence
                 FROM narratives n
                 JOIN images i ON n.image_id = i.id
                 WHERE (n.content LIKE ?1 ESCAPE '\\' OR n.entities_json LIKE ?1 ESCAPE '\\')
                 AND i.id NOT IN ({})
                 ORDER BY n.created_at DESC
                 LIMIT ?2",
                placeholders.join(", ")
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(AppError::database)?;

        let narrative_results: Vec<SearchResult> = if existing_ids.is_empty() {
            let rows = stmt
                .query_map(rusqlite::params![like_pattern, remaining as i64], |row| {
                    Ok(SearchResult {
                        image_id: row.get(0)?,
                        file_path: row.get(1)?,
                        file_name: row.get(2)?,
                        thumbnail_path: row.get(3)?,
                        ai_description: row.get(4)?,
                        ai_tags: row.get(5)?,
                        ai_category: row.get(6)?,
                        ai_confidence: row.get(7)?,
                        match_count: 1,
                        relevance_score: 0.5,
                    })
                })
                .map_err(AppError::database)?;

            rows.filter_map(|r| r.ok()).collect()
        } else {
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
                vec![Box::new(like_pattern), Box::new(remaining as i64)];
            for id in &existing_ids {
                params.push(Box::new(*id));
            }
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt
                .query_map(&param_refs[..], |row| {
                    Ok(SearchResult {
                        image_id: row.get(0)?,
                        file_path: row.get(1)?,
                        file_name: row.get(2)?,
                        thumbnail_path: row.get(3)?,
                        ai_description: row.get(4)?,
                        ai_tags: row.get(5)?,
                        ai_category: row.get(6)?,
                        ai_confidence: row.get(7)?,
                        match_count: 1,
                        relevance_score: 0.5,
                    })
                })
                .map_err(AppError::database)?;

            rows.filter_map(|r| r.ok()).collect()
        };

        results.extend(narrative_results);
    }

    let total = results.len();

    Ok(SearchResponse {
        results,
        total,
        page,
        page_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search_cmd.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_search_empty_query() {
        let request = SearchRequest {
            query: "".to_string(),
            category: None,
            tags: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
        };

        assert!(request.query.trim().is_empty());
    }

    #[test]
    fn test_search_request_defaults() {
        let request = SearchRequest {
            query: "cat".to_string(),
            category: None,
            tags: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
        };

        assert_eq!(request.page.unwrap_or(0), 0);
        assert_eq!(request.page_size.unwrap_or(20), 20);
    }

    #[test]
    fn test_search_response_structure() {
        let response = SearchResponse {
            results: vec![],
            total: 0,
            page: 0,
            page_size: 20,
        };

        assert!(response.results.is_empty());
        assert_eq!(response.total, 0);
        assert_eq!(response.page, 0);
        assert_eq!(response.page_size, 20);
    }

    #[test]
    fn test_search_with_filters() {
        let request = SearchRequest {
            query: "mountain landscape".to_string(),
            category: Some("landscape".to_string()),
            tags: Some(vec!["nature".to_string(), "outdoor".to_string()]),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            page: Some(1),
            page_size: Some(10),
        };

        assert_eq!(request.query, "mountain landscape");
        assert_eq!(request.category, Some("landscape".to_string()));
        assert_eq!(request.tags.as_ref().unwrap().len(), 2);
        assert_eq!(request.page, Some(1));
        assert_eq!(request.page_size, Some(10));
    }

    #[test]
    fn test_search_pagination_calculation() {
        let page = 2u32;
        let page_size = 15u32;
        let offset = (page * page_size) as usize;

        assert_eq!(offset, 30);
    }
}
