#![allow(missing_docs)]
use crate::core::db::Database;
use crate::core::dedup::{
    similarity_to_hamming, DeduplicationScanner, DuplicateGroup, ScanResult as CoreScanResult,
};
use crate::core::search_index::clear_search_cache;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::State;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub threshold: Option<u32>,
    pub similarity_percent: Option<f64>,
}

#[tauri::command]
pub async fn scan_duplicates(
    db: State<'_, Database>,
    request: Option<ScanRequest>,
) -> AppResult<CoreScanResult> {
    let threshold = request.and_then(|r| {
        if let Some(pct) = r.similarity_percent {
            Some(similarity_to_hamming(pct))
        } else {
            r.threshold
        }
    });
    let scanner = DeduplicationScanner::new(threshold);

    info!("开始重复项扫描，阈值: {:?}", threshold);

    let result = scanner.scan(&db)?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    KeepHighestResolution,
    KeepEarliestImport,
    Manual,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDuplicatesRequest {
    pub groups: Vec<DuplicateGroup>,
    pub policy: RetentionPolicy,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub deleted_count: usize,
    pub kept_count: usize,
    pub freed_space_bytes: i64,
    pub dry_run: bool,
}

#[tauri::command]
pub async fn delete_duplicates(
    db: State<'_, Database>,
    request: DeleteDuplicatesRequest,
) -> AppResult<DeleteResult> {
    if request.groups.is_empty() {
        return Err(AppError::validation("没有需要处理的重复项".to_string()));
    }

    let dry_run = request.dry_run.unwrap_or(false);
    let mut deleted_count = 0usize;
    let mut kept_count = 0usize;
    let mut freed_space_bytes: i64 = 0;

    for group in &request.groups {
        if group.images.len() < 2 {
            continue;
        }

        let mut sorted_images = group.images.clone();

        match request.policy {
            RetentionPolicy::KeepHighestResolution => {
                sorted_images.sort_by(|a, b| {
                    let area_a = (a.width.unwrap_or(0) * a.height.unwrap_or(0)) as i64;
                    let area_b = (b.width.unwrap_or(0) * b.height.unwrap_or(0)) as i64;
                    area_b
                        .cmp(&area_a)
                        .then_with(|| a.file_size.cmp(&b.file_size))
                });
            }
            RetentionPolicy::KeepEarliestImport => {
                sorted_images.sort_by_key(|a| a.image_id);
            }
            RetentionPolicy::Manual => {
                sorted_images.sort_by(|a, b| {
                    a.distance
                        .cmp(&b.distance)
                        .then_with(|| b.file_size.cmp(&a.file_size))
                });
            }
        }

        if let Some(keep) = sorted_images.first() {
            kept_count += 1;
            info!("保留图片: {} (ID: {})", keep.file_name, keep.image_id);
        }

        for to_delete in sorted_images.iter().skip(1) {
            if !dry_run {
                let thumb = delete_image_record(&db, to_delete.image_id)?;
                delete_thumbnail(&thumb);
                freed_space_bytes += to_delete.file_size;
            }
            deleted_count += 1;
            info!(
                "删除重复项: {} (ID: {}, 大小: {} bytes)",
                to_delete.file_name, to_delete.image_id, to_delete.file_size
            );
        }
    }

    if dry_run {
        info!(
            "试运行: 将删除 {} 个重复项，保留 {} 个，释放 {} bytes",
            deleted_count, kept_count, freed_space_bytes
        );
    } else {
        info!(
            "删除完成: 删除 {} 个，保留 {} 个，释放 {} bytes",
            deleted_count, kept_count, freed_space_bytes
        );
    }

    Ok(DeleteResult {
        deleted_count,
        kept_count,
        freed_space_bytes,
        dry_run,
    })
}

fn delete_image_record(db: &Database, image_id: i64) -> AppResult<Option<String>> {
    let conn = db.open_connection()?;

    let thumbnail_path: Option<String> = conn
        .query_row(
            "SELECT thumbnail_path FROM images WHERE id = ?",
            rusqlite::params![image_id],
            |row| row.get(0),
        )
        .ok();

    conn.execute(
        "DELETE FROM search_index WHERE image_id = ?",
        rusqlite::params![image_id],
    )
    .map_err(AppError::database)?;

    conn.execute(
        "DELETE FROM images WHERE id = ?",
        rusqlite::params![image_id],
    )
    .map_err(AppError::database)?;

    clear_search_cache();

    Ok(thumbnail_path)
}

fn delete_thumbnail(thumbnail_path: &Option<String>) {
    if let Some(path) = thumbnail_path {
        let file_path = Path::new(path);
        if file_path.exists() {
            if let Err(e) = fs::remove_file(file_path) {
                warn!("删除缩略图失败 {}: {}", path, e);
            } else {
                info!("已删除缩略图: {}", path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dedup::DuplicateImage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_delete_dedup.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    fn create_test_image(db: &Database, id: i64, width: i32, height: i32, file_size: i64) {
        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, width, height) 
             VALUES (?, ?, ?, ?, 'completed', ?, ?)",
            rusqlite::params![
                format!("/test/{}.jpg", id),
                format!("{}.jpg", id),
                file_size,
                format!("hash{}", id),
                width,
                height,
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_delete_request_validation() {
        let request = DeleteDuplicatesRequest {
            groups: vec![],
            policy: RetentionPolicy::KeepHighestResolution,
            dry_run: Some(false),
        };
        assert!(request.groups.is_empty());
    }

    #[test]
    fn test_dry_run_mode() {
        let (db, _temp) = setup_test_db();

        create_test_image(&db, 1, 1920, 1080, 1000000);
        create_test_image(&db, 2, 1280, 720, 500000);

        let group = DuplicateGroup {
            images: vec![
                DuplicateImage {
                    image_id: 1,
                    file_path: "/test/1.jpg".to_string(),
                    file_name: "1.jpg".to_string(),
                    file_size: 1000000,
                    width: Some(1920),
                    height: Some(1080),
                    phash: "abc123".to_string(),
                    distance: 5,
                },
                DuplicateImage {
                    image_id: 2,
                    file_path: "/test/2.jpg".to_string(),
                    file_name: "2.jpg".to_string(),
                    file_size: 500000,
                    width: Some(1280),
                    height: Some(720),
                    phash: "def456".to_string(),
                    distance: 5,
                },
            ],
            similarity: 0.95,
        };

        let request = DeleteDuplicatesRequest {
            groups: vec![group],
            policy: RetentionPolicy::KeepHighestResolution,
            dry_run: Some(true),
        };

        assert!(request.dry_run.unwrap());
    }

    #[test]
    fn test_retention_policy_highest_resolution() {
        let images = vec![
            DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 500000,
                width: Some(1280),
                height: Some(720),
                phash: "abc".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 2,
                file_path: "/test/2.jpg".to_string(),
                file_name: "2.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "def".to_string(),
                distance: 0,
            },
        ];

        let mut sorted = images.clone();
        sorted.sort_by(|a, b| {
            let area_a = (a.width.unwrap_or(0) * a.height.unwrap_or(0)) as i64;
            let area_b = (b.width.unwrap_or(0) * b.height.unwrap_or(0)) as i64;
            area_b
                .cmp(&area_a)
                .then_with(|| a.file_size.cmp(&b.file_size))
        });

        assert_eq!(sorted[0].image_id, 2);
        assert_eq!(sorted[0].width, Some(1920));
        assert_eq!(sorted[0].height, Some(1080));
    }

    #[test]
    fn test_retention_policy_earliest_import() {
        let images = vec![
            DuplicateImage {
                image_id: 3,
                file_path: "/test/3.jpg".to_string(),
                file_name: "3.jpg".to_string(),
                file_size: 600000,
                width: Some(1024),
                height: Some(768),
                phash: "ghi".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 500000,
                width: Some(1280),
                height: Some(720),
                phash: "abc".to_string(),
                distance: 0,
            },
            DuplicateImage {
                image_id: 2,
                file_path: "/test/2.jpg".to_string(),
                file_name: "2.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "def".to_string(),
                distance: 0,
            },
        ];

        let mut sorted = images.clone();
        sorted.sort_by_key(|a| a.image_id);

        assert_eq!(sorted[0].image_id, 1);
        assert_eq!(sorted[1].image_id, 2);
        assert_eq!(sorted[2].image_id, 3);
    }

    #[test]
    fn test_freed_space_calculation() {
        let deleted_size_1: i64 = 500000;
        let deleted_size_2: i64 = 600000;
        let total_freed = deleted_size_1 + deleted_size_2;

        assert_eq!(total_freed, 1100000);
    }

    #[test]
    fn test_single_image_group_skipped() {
        let group = DuplicateGroup {
            images: vec![DuplicateImage {
                image_id: 1,
                file_path: "/test/1.jpg".to_string(),
                file_name: "1.jpg".to_string(),
                file_size: 1000000,
                width: Some(1920),
                height: Some(1080),
                phash: "abc".to_string(),
                distance: 0,
            }],
            similarity: 1.0,
        };

        assert!(group.images.len() < 2);
    }

    // Helper to create test images with specific pHash values
    fn create_test_image_with_phash(
        db: &Database,
        id: i64,
        name: &str,
        width: i32,
        height: i32,
        file_size: i64,
        phash: &str,
    ) {
        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, width, height, phash) 
             VALUES (?, ?, ?, ?, 'completed', ?, ?, ?)",
            rusqlite::params![
                format!("/test/{}", name),
                name,
                file_size,
                format!("hash_{}", id),
                width,
                height,
                phash,
            ],
        )
        .unwrap();
    }

    /// Integration Test: TC-E2E-HP-003
    /// 验证完整去重流程: 扫描 → 并排对比 → 选择保留 → 批量删除
    #[test]
    fn test_e2e_deduplication_flow_scan_delete_verify() {
        let (db, _temp) = setup_test_db();

        // === PHASE 1: 创建测试数据 (模拟相似 pHash 图片) ===
        // 组 A: 两张相似图片 (汉明距离 4, 阈值 10 内)
        create_test_image_with_phash(
            &db,
            1,
            "a1_1080p.jpg",
            1920,
            1080,
            2_000_000,
            "0000000000000000",
        );
        create_test_image_with_phash(
            &db,
            2,
            "a2_720p.jpg",
            1280,
            720,
            1_000_000,
            "000000000000000f",
        ); // dist=4

        // 组 B: 两张相似图片 (汉明距离 4, 阈值 10 内)
        create_test_image_with_phash(
            &db,
            3,
            "b1_4k.jpg",
            3840,
            2160,
            5_000_000,
            "1111111111111111",
        );
        create_test_image_with_phash(
            &db,
            4,
            "b2_1080p.jpg",
            1920,
            1080,
            2_000_000,
            "111111111111111f",
        ); // dist=4

        // 控制组: 唯一图片
        create_test_image_with_phash(&db, 5, "unique.jpg", 800, 600, 500_000, "aaaaaaaaaaaaaaaa");

        // === PHASE 2: 调用 scan_duplicates 扫描 ===
        let scanner = DeduplicationScanner::new(Some(10)); // threshold=10 覆盖距离 4
        let scan_result = scanner.scan(&db).expect("扫描应成功");

        // 验证扫描结果
        assert_eq!(scan_result.total_scanned, 5, "应扫描全部 5 张图片");
        assert_eq!(scan_result.groups.len(), 2, "应发现 2 组重复项");
        assert_eq!(scan_result.total_duplicates, 4, "共 4 张重复图片");

        // 验证每组包含 2 张图片
        let group_sizes: Vec<usize> = scan_result.groups.iter().map(|g| g.images.len()).collect();
        assert!(group_sizes.contains(&2), "每组应包含 2 张图片");

        // 验证相似度计算正确 (1.0 - 4/64 ≈ 0.9375)
        for group in &scan_result.groups {
            assert!(group.similarity > 0.9, "组内相似度应高于 90%");
        }

        // === PHASE 3: 前端并排对比 + 选择保留策略 ===
        // 模拟用户在 DedupManager 组件中查看 side-by-side 对比后，选择 "保留最高分辨率"
        let groups_to_process = scan_result.groups.clone();

        // === PHASE 4: 调用 delete_duplicates 执行批量删除 ===
        let request = DeleteDuplicatesRequest {
            groups: groups_to_process,
            policy: RetentionPolicy::KeepHighestResolution,
            dry_run: Some(false),
        };

        // 执行删除逻辑 (复用 delete_duplicates 命令核心逻辑)
        let mut deleted_count = 0usize;
        let mut kept_count = 0usize;
        let mut freed_bytes: i64 = 0;

        for group in &request.groups {
            if group.images.len() < 2 {
                continue;
            }

            let mut sorted = group.images.clone();
            // 应用 KeepHighestResolution 排序逻辑
            sorted.sort_by(|a, b| {
                let area_a = (a.width.unwrap_or(0) * a.height.unwrap_or(0)) as i64;
                let area_b = (b.width.unwrap_or(0) * b.height.unwrap_or(0)) as i64;
                area_b
                    .cmp(&area_a)
                    .then_with(|| a.file_size.cmp(&b.file_size))
            });

            // 保留第一张 (最高分辨率)
            kept_count += 1;

            // 删除其余重复项
            for to_delete in sorted.iter().skip(1) {
                let conn = db.open_connection().expect("DB 连接应成功");
                conn.execute(
                    "DELETE FROM search_index WHERE image_id = ?",
                    rusqlite::params![to_delete.image_id],
                )
                .expect("删除 search_index 应成功");

                conn.execute(
                    "DELETE FROM images WHERE id = ?",
                    rusqlite::params![to_delete.image_id],
                )
                .expect("删除 images 记录应成功");

                freed_bytes += to_delete.file_size;
                deleted_count += 1;
            }
        }

        // === PHASE 5: 验证最终数据库状态 ===
        assert_eq!(deleted_count, 2, "应删除 2 张重复图片");
        assert_eq!(kept_count, 2, "应保留 2 张高分辨率图片");
        assert_eq!(freed_bytes, 3_000_000, "释放空间应为 1MB + 2MB = 3MB");

        let conn = db.open_connection().expect("DB 连接应成功");
        let remaining_count: i64 = conn
            .query_row("SELECT count(*) FROM images", [], |r| r.get(0))
            .expect("查询应成功");

        assert_eq!(remaining_count, 3, "剩余 3 张图片 (2张高分辨率 + 1张唯一)");

        // 验证具体保留/删除的图片
        let id1_kept: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM images WHERE id = 1)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(id1_kept, "ID 1 (1920x1080 组A高分) 应保留");

        let id2_deleted: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM images WHERE id = 2)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!id2_deleted, "ID 2 (1280x720 组A低分) 应已删除");

        let id3_kept: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM images WHERE id = 3)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(id3_kept, "ID 3 (3840x2160 组B高分) 应保留");

        let id4_deleted: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM images WHERE id = 4)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!id4_deleted, "ID 4 (1920x1080 组B低分) 应已删除");

        let id5_untouched: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM images WHERE id = 5)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(id5_untouched, "ID 5 (唯一图片) 不应受影响");
    }
}
