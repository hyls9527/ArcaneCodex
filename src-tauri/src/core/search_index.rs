#![allow(missing_docs)]
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use jieba_rs::Jieba;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};

static GLOBAL_JIEBA: OnceLock<Jieba> = OnceLock::new();

fn global_jieba() -> &'static Jieba {
    GLOBAL_JIEBA.get_or_init(Jieba::new)
}

const SEARCH_CACHE_TTL: Duration = Duration::from_secs(300);
const MAX_CACHE_ENTRIES: usize = 1000;
const CACHE_EVICT_BATCH: usize = 100;

type SearchCacheStore = Mutex<HashMap<u64, (Instant, Vec<SearchResult>)>>;

static SEARCH_CACHE: OnceLock<SearchCacheStore> = OnceLock::new();

fn search_cache() -> &'static Mutex<HashMap<u64, (Instant, Vec<SearchResult>)>> {
    SEARCH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear_search_cache() {
    if let Some(cache) = SEARCH_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            let count = guard.len();
            guard.clear();
            info!("Search cache cleared ({} entries)", count);
        }
    }
}

fn make_cache_key(
    query: &str,
    filters: Option<&SearchFilters>,
    limit: usize,
    offset: usize,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.hash(&mut hasher);
    limit.hash(&mut hasher);
    offset.hash(&mut hasher);
    if let Some(f) = filters {
        f.category.hash(&mut hasher);
        if let Some(ref tags) = f.tags {
            for t in tags {
                t.hash(&mut hasher);
            }
        }
        f.start_date.hash(&mut hasher);
        f.end_date.hash(&mut hasher);
    }
    hasher.finish()
}

const STOP_WORDS: &[&str] = &[
    "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也",
    "很", "到", "说", "要", "去", "你", "会", "着", "与", "把", "a", "an", "the", "is", "are",
    "was", "were", "be", "been", "being", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "into", "and", "or", "but", "not", "no", "all", "any", "both", "each", "few",
    "more", "most", "other", "some", "such", "that", "this", "these", "those", "it", "its", "this",
    "these", "those", "what", "which", "who", "whom",
];

pub struct SearchIndexBuilder;

impl Default for SearchIndexBuilder {
    fn default() -> Self {
        Self
    }
}

impl SearchIndexBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let words = global_jieba().cut(text, false);

        words
            .iter()
            .map(|w| w.to_lowercase().trim().to_string())
            .filter(|w| {
                !w.is_empty()
                    && w.len() > 1
                    && !STOP_WORDS.contains(&w.as_str())
                    && !w.chars().all(|c| c.is_ascii_punctuation())
            })
            .collect()
    }

    pub fn build_for_image(
        &self,
        db: &Database,
        image_id: i64,
        ai_description: &str,
        ai_tags: &[String],
        ai_category: &str,
    ) -> AppResult<()> {
        let mut conn = db.open_connection()?;

        conn.execute(
            "DELETE FROM search_index WHERE image_id = ?",
            rusqlite::params![image_id],
        )
        .map_err(AppError::database)?;

        let mut tokens = Vec::new();
        tokens.extend(self.tokenize(ai_description));
        tokens.extend(ai_tags.iter().map(|t| t.to_lowercase()));
        tokens.extend(self.tokenize(ai_category));
        tokens.extend(self.tokenize(&format!("id {}", image_id)));

        tokens.sort();
        tokens.dedup();

        let tx = conn.transaction().map_err(AppError::database)?;

        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO search_index (image_id, term, field, weight) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(AppError::database)?;

            for token in &tokens {
                let weight = if ai_tags.iter().any(|t| t.to_lowercase() == *token) {
                    2.0
                } else {
                    1.0
                };

                let field = "combined";
                stmt.execute(rusqlite::params![image_id, token, field, weight])
                    .map_err(AppError::database)?;
            }
        }

        tx.commit().map_err(AppError::database)?;

        info!("为图片 {} 构建搜索索引: {} 个词条", image_id, tokens.len());

        clear_search_cache();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_for_image(&self, db: &Database, image_id: i64) -> AppResult<()> {
        let conn = db.open_connection()?;

        let deleted = conn
            .execute(
                "DELETE FROM search_index WHERE image_id = ?",
                rusqlite::params![image_id],
            )
            .map_err(AppError::database)?;

        debug!("删除图片 {} 的搜索索引: {} 条记录", image_id, deleted);

        clear_search_cache();

        Ok(())
    }

    pub fn search(
        &self,
        db: &Database,
        query: &str,
        filters: Option<&SearchFilters>,
        limit: usize,
        offset: usize,
    ) -> AppResult<Vec<SearchResult>> {
        let cache_key = make_cache_key(query, filters, limit, offset);
        {
            if let Ok(cache) = search_cache().lock() {
                if let Some((timestamp, results)) = cache.get(&cache_key) {
                    if timestamp.elapsed() < SEARCH_CACHE_TTL {
                        debug!("Search cache hit for query '{}'", query);
                        return Ok(results.clone());
                    }
                }
            }
        }

        let tokens = self.tokenize(query);

        if tokens.is_empty() {
            return Ok(vec![]);
        }

        let conn = db.open_connection()?;

        let placeholders: Vec<String> = tokens.iter().map(|_| "?".to_string()).collect();
        let terms_clause = placeholders.join(", ");

        let mut sql = format!(
            "SELECT i.id, i.file_path, i.file_name, i.thumbnail_path,
                    i.ai_description, i.ai_tags, i.ai_category, i.ai_confidence,
                    COUNT(DISTINCT si.term) as match_count,
                    SUM(si.weight) as relevance_score
             FROM images i
             INNER JOIN search_index si ON i.id = si.image_id
             WHERE si.term IN ({})
             AND i.ai_status = 'completed'",
            terms_clause
        );

        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = tokens
            .iter()
            .map(|t| Box::new(t.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();

        if let Some(f) = filters {
            if let Some(ref _category) = f.category {
                sql.push_str(" AND i.ai_category = ?");
                params.push(Box::new(_category.clone()));
            }
            if let Some(ref tags) = f.tags {
                for _tag in tags {
                    sql.push_str(" AND i.ai_tags LIKE ? ESCAPE '\\'");
                }
                for tag in tags {
                    let escaped_tag = tag
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_");
                    params.push(Box::new(format!("%{}%", escaped_tag)));
                }
            }
            if let Some(ref _start_date) = f.start_date {
                sql.push_str(" AND i.created_at >= ?");
                params.push(Box::new(_start_date.clone()));
            }
            if let Some(ref _end_date) = f.end_date {
                sql.push_str(" AND i.created_at <= ?");
                params.push(Box::new(_end_date.clone()));
            }
        }

        sql.push_str(
            " GROUP BY i.id 
             ORDER BY relevance_score DESC, match_count DESC, i.created_at DESC
             LIMIT ? OFFSET ?",
        );

        params.push(Box::new(limit as i64));
        params.push(Box::new(offset as i64));

        let mut stmt = conn.prepare(&sql).map_err(AppError::database)?;

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
                    match_count: row.get(8)?,
                    relevance_score: row.get(9)?,
                })
            })
            .map_err(AppError::database)?;

        let results: Vec<SearchResult> = rows
            .filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => {
                    debug!("读取搜索结果失败: {}", e);
                    None
                }
            })
            .collect();

        info!("搜索 '{}' 返回 {} 条结果", query, results.len());

        {
            if let Ok(mut cache) = search_cache().lock() {
                // Purge expired entries (TTL-based eviction)
                cache.retain(|_, (timestamp, _)| timestamp.elapsed() < SEARCH_CACHE_TTL);

                // If the cache is still over capacity, evict the oldest batch
                if cache.len() > MAX_CACHE_ENTRIES {
                    let mut entries: Vec<(u64, Instant)> = cache
                        .iter()
                        .map(|(k, (ts, _))| (*k, *ts))
                        .collect();
                    entries.sort_by_key(|(_, ts)| *ts);
                    for key in entries.iter().take(CACHE_EVICT_BATCH).map(|(k, _)| *k) {
                        cache.remove(&key);
                    }
                }

                cache.insert(cache_key, (Instant::now(), results.clone()));
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub thumbnail_path: Option<String>,
    pub ai_description: Option<String>,
    pub ai_tags: Option<String>,
    pub ai_category: Option<String>,
    pub ai_confidence: Option<f64>,
    pub match_count: i64,
    pub relevance_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_tokenize_chinese() {
        let builder = SearchIndexBuilder::new();
        let tokens = builder.tokenize("这是一只可爱的小猫在草地上玩耍");

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.contains("可爱")));
        assert!(tokens.iter().any(|t| t.contains("猫")));
    }

    #[test]
    fn test_tokenize_english() {
        let builder = SearchIndexBuilder::new();
        let tokens = builder.tokenize("A beautiful sunset over the ocean");

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.contains("beautiful")));
        assert!(tokens.iter().any(|t| t.contains("sunset")));
        assert!(tokens.iter().any(|t| t.contains("ocean")));
        assert!(!tokens.iter().any(|t| t == "a"));
        assert!(!tokens.iter().any(|t| t == "the"));
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let builder = SearchIndexBuilder::new();

        let tokens = builder.tokenize("这是一个非常美丽的风景");
        assert!(!tokens.contains(&"的".to_string()));
        assert!(!tokens.contains(&"一".to_string()));
        assert!(!tokens.contains(&"是".to_string()));
    }

    #[test]
    fn test_tokenize_lowercase() {
        let builder = SearchIndexBuilder::new();
        let tokens = builder.tokenize("Beautiful SUNSET over THE Ocean");

        assert!(tokens.iter().any(|t| t == "beautiful"));
        assert!(tokens.iter().any(|t| t == "sunset"));
        assert!(tokens.iter().any(|t| t == "ocean"));
    }

    #[test]
    fn test_tokenize_empty_string() {
        let builder = SearchIndexBuilder::new();
        let tokens = builder.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_punctuation_only() {
        let builder = SearchIndexBuilder::new();
        let tokens = builder.tokenize("... !!! ???");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_build_and_search_index() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/cat.jpg', 'cat.jpg', 1000, 'abc123', 'completed', 'A cute orange cat sleeping on a sofa', 'cat,animal,pet,cute,feline', 'animal')",
            [],
        )
        .unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'abc123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["cat", "animal", "pet", "cute", "feline"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let result = builder.build_for_image(
            &db,
            image_id,
            "A cute orange cat sleeping on a sofa",
            &tags,
            "animal",
        );
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE image_id = ?",
                rusqlite::params![image_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count > 0, "搜索索引应该有记录");

        let results = builder.search(&db, "cute cat", None, 10, 0).unwrap();
        assert!(!results.is_empty(), "搜索 'cute cat' 应该返回结果");
        assert!(results[0].relevance_score > 0.0);
    }

    #[test]
    fn test_delete_search_index() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/del.jpg', 'del.jpg', 1000, 'del123', 'completed', 'Test image', 'test,image', 'object')",
            [],
        )
        .unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'del123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["test", "image"].iter().map(|s| s.to_string()).collect();

        builder
            .build_for_image(&db, image_id, "Test image", &tags, "object")
            .unwrap();

        let count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE image_id = ?",
                rusqlite::params![image_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count_before > 0);

        builder.delete_for_image(&db, image_id).unwrap();

        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE image_id = ?",
                rusqlite::params![image_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_after, 0);
    }

    #[test]
    fn test_search_with_filters() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/landscape.jpg', 'landscape.jpg', 1000, 'land123', 'completed', 'Beautiful mountain landscape with snow', 'mountain,landscape,nature,snow', 'landscape')",
            [],
        )
        .unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'land123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["mountain", "landscape", "nature", "snow"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        builder
            .build_for_image(
                &db,
                image_id,
                "Beautiful mountain landscape with snow",
                &tags,
                "landscape",
            )
            .unwrap();

        let filters = SearchFilters {
            category: Some("landscape".to_string()),
            tags: None,
            start_date: None,
            end_date: None,
        };

        let results = builder
            .search(&db, "mountain snow", Some(&filters), 10, 0)
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].ai_category, Some("landscape".to_string()));
    }

    #[test]
    fn test_search_cache_hit() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/cache.jpg', 'cache.jpg', 1000, 'cache123', 'completed', 'A red car on the road', 'car,vehicle,red,road', 'object')",
            [],
        ).unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'cache123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["car", "vehicle", "red", "road"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        builder
            .build_for_image(&db, image_id, "A red car on the road", &tags, "object")
            .unwrap();

        clear_search_cache();

        let results1 = builder.search(&db, "red car", None, 10, 0).unwrap();
        assert!(!results1.is_empty());

        let results2 = builder.search(&db, "red car", None, 10, 0).unwrap();
        assert_eq!(results1.len(), results2.len());
        assert_eq!(results1[0].image_id, results2[0].image_id);
    }

    #[test]
    fn test_search_cache_invalidation_on_build() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/inv1.jpg', 'inv1.jpg', 1000, 'inv123', 'completed', 'A blue sky', 'sky,blue', 'landscape')",
            [],
        ).unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'inv123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["sky", "blue"].iter().map(|s| s.to_string()).collect();
        builder
            .build_for_image(&db, image_id, "A blue sky", &tags, "landscape")
            .unwrap();

        clear_search_cache();

        let results1 = builder.search(&db, "blue sky", None, 10, 0).unwrap();
        assert!(!results1.is_empty());

        builder
            .build_for_image(
                &db,
                image_id,
                "Updated description with clouds",
                &tags,
                "landscape",
            )
            .unwrap();

        let results2 = builder.search(&db, "blue sky", None, 10, 0).unwrap();
        assert!(!results2.is_empty());
    }

    #[test]
    fn test_search_cache_different_queries() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/multi1.jpg', 'multi1.jpg', 1000, 'multi123', 'completed', 'A green forest', 'forest,green,nature', 'landscape')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/multi2.jpg', 'multi2.jpg', 1000, 'multi456', 'completed', 'A busy city street', 'city,street,busy', 'cityscape')",
            [],
        ).unwrap();

        let id1: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'multi123'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let id2: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'multi456'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags1: Vec<String> = ["forest", "green", "nature"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let tags2: Vec<String> = ["city", "street", "busy"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        builder
            .build_for_image(&db, id1, "A green forest", &tags1, "landscape")
            .unwrap();
        builder
            .build_for_image(&db, id2, "A busy city street", &tags2, "cityscape")
            .unwrap();

        clear_search_cache();

        let forest_results = builder.search(&db, "green forest", None, 10, 0).unwrap();
        let city_results = builder.search(&db, "busy city", None, 10, 0).unwrap();

        assert!(!forest_results.is_empty());
        assert!(!city_results.is_empty());
        assert_ne!(forest_results[0].image_id, city_results[0].image_id);
    }

    #[test]
    fn test_clear_search_cache_function() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, ai_description, ai_tags, ai_category) 
             VALUES ('/test/clr.jpg', 'clr.jpg', 1000, 'clr123', 'completed', 'A test image', 'test', 'object')",
            [],
        ).unwrap();

        let image_id: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_hash = 'clr123'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let builder = SearchIndexBuilder::new();
        let tags: Vec<String> = ["test"].iter().map(|s| s.to_string()).collect();
        builder
            .build_for_image(&db, image_id, "A test image", &tags, "object")
            .unwrap();

        clear_search_cache();

        let _results = builder.search(&db, "test", None, 10, 0).unwrap();

        clear_search_cache();

        let results_after_clear = builder.search(&db, "test", None, 10, 0).unwrap();
        assert!(!results_after_clear.is_empty());
    }
}
