#![allow(missing_docs)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Image {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnail_path: Option<String>,
    pub phash: Option<String>,
    pub exif_data: Option<serde_json::Value>,
    pub ai_status: String,
    pub ai_tags: Option<serde_json::Value>,
    pub ai_description: Option<String>,
    pub ai_category: Option<String>,
    pub ai_confidence: Option<f64>,
    pub ai_model: Option<String>,
    pub ai_processed_at: Option<String>,
    pub ai_error_message: Option<String>,
    pub ai_retry_count: i32,
    pub source: String,
    pub generation_source: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
    pub generation_workflow_id: Option<String>,
    pub ai_provider: Option<String>,
    pub ai_tag_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
impl Image {
    pub fn from_row(row: &rusqlite::Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_hash: row.get(4)?,
            mime_type: row.get(5)?,
            width: row.get(6)?,
            height: row.get(7)?,
            thumbnail_path: row.get(8)?,
            phash: row.get(9)?,
            exif_data: row
                .get::<_, Option<String>>(10)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::json!({}))),
            ai_status: row.get(11)?,
            ai_tags: row
                .get::<_, Option<String>>(12)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::json!([]))),
            ai_description: row.get(13)?,
            ai_category: row.get(14)?,
            ai_confidence: row.get(15)?,
            ai_model: row.get(16)?,
            ai_processed_at: row.get(17)?,
            ai_error_message: row.get(18)?,
            ai_retry_count: row.get(19)?,
            source: row.get(20)?,
            generation_source: row.get(21)?,
            generation_metadata: row
                .get::<_, Option<String>>(22)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::json!({}))),
            generation_workflow_id: row.get(23)?,
            ai_provider: row.get(24)?,
            ai_tag_status: row.get(25)?,
            created_at: row.get(26)?,
            updated_at: row.get(27)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::inference::AIResult;
    use crate::core::search_index::SearchResult;

    #[test]
    fn test_image_serialization() {
        let image = Image {
            id: 1,
            file_path: "/test/image.jpg".to_string(),
            file_name: "image.jpg".to_string(),
            file_size: 1000,
            file_hash: Some("hash123".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            width: Some(1920),
            height: Some(1080),
            thumbnail_path: Some("/test/thumb.webp".to_string()),
            phash: Some("abc123".to_string()),
            exif_data: Some(serde_json::json!({"camera": "Canon"})),
            ai_status: "completed".to_string(),
            ai_tags: Some(serde_json::json!(["cat", "animal"])),
            ai_description: Some("A cute cat".to_string()),
            ai_category: Some("animal".to_string()),
            ai_confidence: Some(0.95),
            ai_model: Some("Qwen2.5-VL-7B-Instruct".to_string()),
            ai_processed_at: Some("2024-01-01 12:00:00".to_string()),
            ai_error_message: None,
            ai_retry_count: 0,
            source: "import".to_string(),
            generation_source: None,
            generation_metadata: None,
            generation_workflow_id: None,
            ai_provider: Some("lm_studio".to_string()),
            ai_tag_status: Some("verified".to_string()),
            created_at: "2024-01-01 12:00:00".to_string(),
            updated_at: "2024-01-01 12:00:00".to_string(),
        };

        let json = serde_json::to_string(&image).unwrap();
        let deserialized: Image = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.file_name, "image.jpg");
        assert_eq!(deserialized.ai_status, "completed");
        assert_eq!(deserialized.ai_confidence, Some(0.95));
    }

    #[test]
    fn test_image_with_null_fields() {
        let image = Image {
            id: 1,
            file_path: "/test/image.jpg".to_string(),
            file_name: "image.jpg".to_string(),
            file_size: 1000,
            file_hash: None,
            mime_type: None,
            width: None,
            height: None,
            thumbnail_path: None,
            phash: None,
            exif_data: None,
            ai_status: "pending".to_string(),
            ai_tags: None,
            ai_description: None,
            ai_category: None,
            ai_confidence: None,
            ai_model: None,
            ai_processed_at: None,
            ai_error_message: None,
            ai_retry_count: 0,
            source: "import".to_string(),
            generation_source: None,
            generation_metadata: None,
            generation_workflow_id: None,
            ai_provider: None,
            ai_tag_status: None,
            created_at: "2024-01-01 12:00:00".to_string(),
            updated_at: "2024-01-01 12:00:00".to_string(),
        };

        let json = serde_json::to_string(&image).unwrap();
        let deserialized: Image = serde_json::from_str(&json).unwrap();

        assert!(deserialized.file_hash.is_none());
        assert!(deserialized.ai_tags.is_none());
        assert!(deserialized.ai_confidence.is_none());
    }

    #[test]
    fn test_ai_result_serialization() {
        let result = AIResult {
            tags: vec!["cat".to_string(), "animal".to_string()],
            description: "A cute cat sleeping on a sofa".to_string(),
            category: "animal".to_string(),
            confidence: 0.92,
            raw_response: "{}".to_string(),
            provider: "lm_studio".to_string(),
            model: "test-model".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AIResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tags.len(), 2);
        assert_eq!(deserialized.category, "animal");
        assert_eq!(deserialized.confidence, 0.92);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            image_id: 123,
            file_path: "/test/image.jpg".to_string(),
            file_name: "image.jpg".to_string(),
            thumbnail_path: Some("/test/thumb.webp".to_string()),
            ai_description: Some("test description".to_string()),
            ai_tags: Some(r#"["tag1","tag2"]"#.to_string()),
            ai_category: Some("animal".to_string()),
            ai_confidence: Some(0.9),
            match_count: 5,
            relevance_score: 85.0,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"image_id\":123"));
        assert!(json.contains("\"match_count\":5"));
        assert!(json.contains("relevance_score"));
    }
}
