#![allow(missing_docs)]
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct SampleDataStatus {
    pub has_sample_data: bool,
    pub sample_count: i64,
}

struct SampleEntry {
    file_path: &'static str,
    file_name: &'static str,
    file_size: i64,
    file_hash: &'static str,
    mime_type: &'static str,
    width: i32,
    height: i32,
    ai_status: &'static str,
    ai_tags: &'static str,
    ai_description: &'static str,
    ai_category: &'static str,
    ai_confidence: f64,
    ai_model: &'static str,
    ai_provider: &'static str,
    ai_tag_status: &'static str,
    source: &'static str,
}

const SAMPLE_ENTRIES: &[SampleEntry] = &[
    SampleEntry {
        file_path: "/sample/sunset_beach.jpg",
        file_name: "sunset_beach.jpg",
        file_size: 3_245_678,
        file_hash: "sample_sha256_sunset_beach_001",
        mime_type: "image/jpeg",
        width: 3840,
        height: 2160,
        ai_status: "completed",
        ai_tags: r#"["日落","sunset","海滩","beach","海浪","waves","金色","golden","地平线","horizon","晚霞","afterglow"]"#,
        ai_description: "日落时分的海滩，金色阳光洒在波浪上，远处地平线与晚霞交融",
        ai_category: "风景",
        ai_confidence: 0.94,
        ai_model: "Qwen2.5-VL-7B-Instruct",
        ai_provider: "lm_studio",
        ai_tag_status: "verified",
        source: "sample",
    },
    SampleEntry {
        file_path: "/sample/portrait_smile.jpg",
        file_name: "portrait_smile.jpg",
        file_size: 2_876_543,
        file_hash: "sample_sha256_portrait_smile_002",
        mime_type: "image/jpeg",
        width: 2400,
        height: 3200,
        ai_status: "completed",
        ai_tags: r#"["人像","portrait","微笑","smile","女性","woman","户外","outdoor","自然光","natural light"]"#,
        ai_description: "一位女性在户外微笑的特写人像，自然光下表情温暖",
        ai_category: "人像",
        ai_confidence: 0.89,
        ai_model: "Qwen2.5-VL-7B-Instruct",
        ai_provider: "lm_studio",
        ai_tag_status: "provisional",
        source: "sample",
    },
    SampleEntry {
        file_path: "/sample/city_skyline.jpg",
        file_name: "city_skyline.jpg",
        file_size: 4_567_890,
        file_hash: "sample_sha256_city_skyline_003",
        mime_type: "image/jpeg",
        width: 4096,
        height: 2304,
        ai_status: "completed",
        ai_tags: r#"["城市","city","天际线","skyline","建筑","architecture","摩天楼","skyscraper","夜景","night view","灯光","lights"]"#,
        ai_description: "城市天际线夜景，摩天楼的灯光映照在江面上",
        ai_category: "建筑",
        ai_confidence: 0.92,
        ai_model: "Qwen2.5-VL-7B-Instruct",
        ai_provider: "lm_studio",
        ai_tag_status: "verified",
        source: "sample",
    },
    SampleEntry {
        file_path: "/sample/ramen_lunch.jpg",
        file_name: "ramen_lunch.jpg",
        file_size: 1_987_654,
        file_hash: "sample_sha256_ramen_lunch_004",
        mime_type: "image/jpeg",
        width: 3024,
        height: 3024,
        ai_status: "completed",
        ai_tags: r#"["美食","food","拉面","ramen","日式","japanese","午餐","lunch","热气","steam","筷子","chopsticks"]"#,
        ai_description: "一碗热气腾腾的日式拉面，配以溏心蛋和叉烧",
        ai_category: "美食",
        ai_confidence: 0.87,
        ai_model: "Qwen2.5-VL-7B-Instruct",
        ai_provider: "lm_studio",
        ai_tag_status: "provisional",
        source: "sample",
    },
    SampleEntry {
        file_path: "/sample/golden_retriever.jpg",
        file_name: "golden_retriever.jpg",
        file_size: 2_345_678,
        file_hash: "sample_sha256_golden_retriever_005",
        mime_type: "image/jpeg",
        width: 3200,
        height: 2400,
        ai_status: "completed",
        ai_tags: r#"["狗","dog","金毛","golden retriever","草地","grass","奔跑","running","宠物","pet","户外","outdoor"]"#,
        ai_description: "一只金毛犬在草地上奔跑，阳光下的毛发闪闪发光",
        ai_category: "动物",
        ai_confidence: 0.96,
        ai_model: "Qwen2.5-VL-7B-Instruct",
        ai_provider: "lm_studio",
        ai_tag_status: "verified",
        source: "sample",
    },
];

pub fn seed_if_empty(db: &Database) -> AppResult<()> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
        .map_err(AppError::database)?;

    if count > 0 {
        info!("Database already has {} images, skipping seed", count);
        return Ok(());
    }

    info!("Empty database detected, seeding sample data...");

    for entry in SAMPLE_ENTRIES {
        conn.execute(
            "INSERT INTO images (
                file_path, file_name, file_size, file_hash, mime_type,
                width, height, ai_status, ai_tags, ai_description,
                ai_category, ai_confidence, ai_model, ai_provider,
                ai_tag_status, source, generation_source
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                entry.file_path,
                entry.file_name,
                entry.file_size,
                entry.file_hash,
                entry.mime_type,
                entry.width,
                entry.height,
                entry.ai_status,
                entry.ai_tags,
                entry.ai_description,
                entry.ai_category,
                entry.ai_confidence,
                entry.ai_model,
                entry.ai_provider,
                entry.ai_tag_status,
                entry.source,
                "sample",
            ],
        )
        .map_err(AppError::database)?;
    }

    info!("Seeded {} sample images", SAMPLE_ENTRIES.len());
    Ok(())
}

#[tauri::command]
pub fn check_sample_data(db: State<'_, Database>) -> AppResult<SampleDataStatus> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let sample_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM images WHERE source = 'sample'",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::database)?;

    Ok(SampleDataStatus {
        has_sample_data: sample_count > 0,
        sample_count,
    })
}

#[tauri::command]
pub fn clear_sample_data(db: State<'_, Database>) -> AppResult<i64> {
    let conn = db.open_connection().map_err(AppError::database)?;

    conn.execute(
        "DELETE FROM search_index WHERE image_id IN (SELECT id FROM images WHERE source = 'sample')",
        [],
    ).map_err(AppError::database)?;

    let deleted = conn
        .execute("DELETE FROM images WHERE source = 'sample'", [])
        .map_err(AppError::database)?;

    info!("Cleared {} sample images", deleted);
    Ok(deleted as i64)
}

#[tauri::command]
pub fn load_sample_data(db: State<'_, Database>) -> AppResult<i64> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM images WHERE source = 'sample'",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::database)?;

    if existing > 0 {
        info!("Sample data already exists ({} images)", existing);
        return Ok(existing);
    }

    let mut inserted = 0i64;
    for entry in SAMPLE_ENTRIES {
        conn.execute(
            "INSERT INTO images (
                file_path, file_name, file_size, file_hash, mime_type,
                width, height, ai_status, ai_tags, ai_description,
                ai_category, ai_confidence, ai_model, ai_provider,
                ai_tag_status, source, generation_source
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                entry.file_path,
                entry.file_name,
                entry.file_size,
                entry.file_hash,
                entry.mime_type,
                entry.width,
                entry.height,
                entry.ai_status,
                entry.ai_tags,
                entry.ai_description,
                entry.ai_category,
                entry.ai_confidence,
                entry.ai_model,
                entry.ai_provider,
                entry.ai_tag_status,
                entry.source,
                "sample",
            ],
        )
        .map_err(AppError::database)?;
        inserted += 1;
    }

    info!("Loaded {} sample images", inserted);
    Ok(inserted)
}
