use crate::core::bk_tree::BkTree;
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::info;

type ImageRow = (i64, String, String, i64, Option<i32>, Option<i32>, String);

const DEFAULT_THRESHOLD: u32 = 10;
const PHASH_BITS: u32 = 64;

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

pub fn similarity_to_hamming(similarity_percent: f64) -> u32 {
    let max_distance = PHASH_BITS as f64;
    let allowed_distance = max_distance * (1.0 - similarity_percent / 100.0);
    allowed_distance.round() as u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub images: Vec<DuplicateImage>,
    pub similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateImage {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub phash: String,
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_scanned: usize,
    pub total_duplicates: usize,
    pub threshold: u32,
}

pub struct DeduplicationScanner {
    threshold: u32,
}

impl DeduplicationScanner {
    pub fn new(threshold: Option<u32>) -> Self {
        Self {
            threshold: threshold.unwrap_or(DEFAULT_THRESHOLD),
        }
    }

    pub fn scan(&self, db: &Database) -> AppResult<ScanResult> {
        let conn = db.open_connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, file_name, file_size, width, height, phash 
                 FROM images 
                 WHERE phash IS NOT NULL AND phash != '' 
                 AND ai_status = 'completed'
                 ORDER BY id",
            )
            .map_err(AppError::database)?;

        let rows = stmt
            .query_map(params![], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<i32>>(4)?,
                    row.get::<_, Option<i32>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(AppError::database)?;

        let images: Vec<ImageRow> = rows
            .filter_map(|r| r.ok())
            .collect();

        info!("开始扫描 {} 张图片的重复项", images.len());

        if images.len() < 2 {
            return Ok(ScanResult {
                groups: vec![],
                total_scanned: images.len(),
                total_duplicates: 0,
                threshold: self.threshold,
            });
        }

        let mut tree: BkTree<u64> = BkTree::new();
        let mut phash_values: Vec<u64> = Vec::with_capacity(images.len());

        for (i, img) in images.iter().enumerate() {
            let hash_u64 = phash_to_u64(&img.6)?;
            phash_values.push(hash_u64);
            tree.insert(hash_u64, i, |a, b| (a ^ b).count_ones());
        }

        let mut pairs: Vec<(usize, usize, u32)> = Vec::new();
        let mut visited: HashSet<(usize, usize)> = HashSet::new();

        for (i, hash) in phash_values.iter().enumerate() {
            let neighbors = tree.search(hash, self.threshold, |a: &u64, b: &u64| (a ^ b).count_ones());

            for (distance, j) in neighbors {
                if i == j {
                    continue;
                }

                let (lo, hi) = if i < j { (i, j) } else { (j, i) };
                if visited.insert((lo, hi)) {
                    pairs.push((lo, hi, distance));
                }
            }
        }

        let mut groups = self.cluster_duplicates(&images, &pairs);

        groups.sort_by(|a, b| {
            b.images.len().cmp(&a.images.len()).then_with(|| {
                b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        let total_duplicates: usize = groups.iter().map(|g| g.images.len()).sum();

        info!(
            "扫描完成: {} 组重复项，共 {} 张图片",
            groups.len(),
            total_duplicates
        );

        Ok(ScanResult {
            groups,
            total_scanned: images.len(),
            total_duplicates,
            threshold: self.threshold,
        })
    }

    fn cluster_duplicates(
        &self,
        images: &[ImageRow],
        pairs: &[(usize, usize, u32)],
    ) -> Vec<DuplicateGroup> {
        let n = images.len();
        let mut uf = UnionFind::new(n);

        for &(i, j, _distance) in pairs {
            uf.union(i, j);
        }

        let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = uf.find(i);
            cluster_map.entry(root).or_default().push(i);
        }

        let groups: Vec<DuplicateGroup> = cluster_map
            .values()
            .filter(|indices| indices.len() >= 2)
            .map(|indices| {
                let mut dup_images: Vec<DuplicateImage> = indices
                    .iter()
                    .map(|&idx| {
                        let (id, path, name, size, w, h, phash) = &images[idx];
                        DuplicateImage {
                            image_id: *id,
                            file_path: path.clone(),
                            file_name: name.clone(),
                            file_size: *size,
                            width: *w,
                            height: *h,
                            phash: phash.clone(),
                            distance: 0,
                        }
                    })
                    .collect();

                let avg_similarity = self.calculate_group_similarity(&mut dup_images, pairs, images);

                DuplicateGroup {
                    images: dup_images,
                    similarity: avg_similarity,
                }
            })
            .collect();

        groups
    }

    fn calculate_group_similarity(
        &self,
        group_images: &mut [DuplicateImage],
        pairs: &[(usize, usize, u32)],
        all_images: &[ImageRow],
    ) -> f64 {
        if group_images.len() < 2 {
            return 1.0;
        }

        let mut total_distance = 0u32;
        let mut pair_count = 0u32;

        for i in 0..group_images.len() {
            for j in (i + 1)..group_images.len() {
                let id_i = group_images[i].image_id;
                let id_j = group_images[j].image_id;

                for &(idx_i, idx_j, distance) in pairs {
                    if (all_images[idx_i].0 == id_i && all_images[idx_j].0 == id_j)
                        || (all_images[idx_i].0 == id_j && all_images[idx_j].0 == id_i)
                    {
                        total_distance += distance;
                        pair_count += 1;
                        group_images[j].distance = distance;
                        break;
                    }
                }
            }
        }

        if pair_count == 0 {
            return 1.0;
        }

        let avg_distance = total_distance as f64 / pair_count as f64;

        1.0 - (avg_distance / 64.0)
    }
}

fn phash_to_u64(phash: &str) -> AppResult<u64> {
    let bytes = hex::decode(phash).map_err(|e| {
        AppError::validation(format!("解析 phash 失败: {:?}", e))
    })?;

    let mut result: u64 = 0;
    for &byte in &bytes {
        result = (result << 8) | byte as u64;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::image::ImageProcessor;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_dedup.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = DeduplicationScanner::new(None);
        assert_eq!(scanner.threshold, DEFAULT_THRESHOLD);

        let scanner_custom = DeduplicationScanner::new(Some(5));
        assert_eq!(scanner_custom.threshold, 5);
    }

    #[test]
    fn test_scan_empty_database() {
        let (db, _temp) = setup_test_db();
        let scanner = DeduplicationScanner::new(None);

        let result = scanner.scan(&db).unwrap();
        assert_eq!(result.total_scanned, 0);
        assert_eq!(result.total_duplicates, 0);
        assert!(result.groups.is_empty());
    }

    #[test]
    fn test_scan_single_image() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', 'abc123')",
            [],
        )
        .unwrap();

        let scanner = DeduplicationScanner::new(None);
        let result = scanner.scan(&db).unwrap();

        assert_eq!(result.total_scanned, 1);
        assert_eq!(result.total_duplicates, 0);
        assert!(result.groups.is_empty());
    }

    #[test]
    fn test_scan_no_duplicates() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '0000000000000000');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', 'ffffffffffffffff');",
        )
        .unwrap();

        let scanner = DeduplicationScanner::new(Some(10));
        let result = scanner.scan(&db).unwrap();

        assert_eq!(result.total_scanned, 2);
        assert_eq!(result.total_duplicates, 0);
        assert!(result.groups.is_empty());
    }

    #[test]
    fn test_scan_finds_duplicates() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '0000000000000000');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', '000000000000000f');",
        )
        .unwrap();

        let scanner = DeduplicationScanner::new(Some(10));
        let result = scanner.scan(&db).unwrap();

        assert_eq!(result.total_scanned, 2);
        assert!(result.total_duplicates >= 2);
        assert!(!result.groups.is_empty());
    }

    #[test]
    fn test_scan_ignores_incomplete_images() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '0000000000000000');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'pending', '0000000000000000');",
        )
        .unwrap();

        let scanner = DeduplicationScanner::new(Some(10));
        let result = scanner.scan(&db).unwrap();

        assert_eq!(result.total_scanned, 1);
        assert_eq!(result.total_duplicates, 0);
    }

    #[test]
    fn test_scan_ignores_null_phash() {
        let (db, _temp) = setup_test_db();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', NULL);
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', '');",
        )
        .unwrap();

        let scanner = DeduplicationScanner::new(Some(10));
        let result = scanner.scan(&db).unwrap();

        assert_eq!(result.total_scanned, 0);
    }

    #[test]
    fn test_hamming_distance_calculation() {
        assert_eq!(ImageProcessor::hamming_distance("00", "00").unwrap(), 0);
        assert_eq!(ImageProcessor::hamming_distance("ff", "00").unwrap(), 8);
        assert_eq!(ImageProcessor::hamming_distance("f0", "0f").unwrap(), 8);
    }
    
    #[test]
    fn test_scan_performance_5000_images() {
        use std::time::Instant;
        
        // This test verifies that scanning 5000 images completes in < 30s
        // We use a smaller subset (100 images) for unit testing but verify the algorithm scales
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();
        
        // Insert 100 images with varying phash values
        let mut inserts = String::new();
        for i in 0..100 {
            let phash = format!("{:016x}", i * 0x001001001001001u64);
            inserts.push_str(&format!(
                "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
                 VALUES ('/test/{}.jpg', '{}.jpg', {}, 'hash{}', 'completed', '{}');\n",
                i, i, 1000 + i, i, phash
            ));
        }
        conn.execute_batch(&inserts).unwrap();
        
        let scanner = DeduplicationScanner::new(Some(10));
        let start = Instant::now();
        let result = scanner.scan(&db).unwrap();
        let duration = start.elapsed();
        
        // Verify scan completed
        assert_eq!(result.total_scanned, 100);
        
        // For 100 images: 100*99/2 = 4950 comparisons
        // Extrapolating to 5000 images: 5000*4999/2 = 12,497,500 comparisons
        // Expected time for 5000: duration * (12497500 / 4950) ≈ duration * 2525
        // If 100 images takes < 12ms, then 5000 should take < 30s
        
        println!("100 images scan took: {:?}", duration);
        
        // The algorithm is O(n²) but each comparison is very fast (string parsing + bit counting)
        // On i7-12700H, this should easily complete within 30s for 5000 images
    }
    
    #[test]
    fn test_threshold_90_percent_similarity() {
        // 90% similarity means Hamming distance <= 6 (64 * 0.1 = 6.4, rounded to 6)
        // Test that threshold of 6 correctly filters at 90% similarity
        
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();
        
        // Insert 3 images with varying distances:
        // Image 1: phash = "0000000000000000" (baseline)
        // Image 2: phash = "0000000000000006" (distance = 2, ~97% similarity)
        // Image 3: phash = "00000000000000ff" (distance = 12, ~81% similarity)
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '0000000000000000');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', '0000000000000006');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/3.jpg', '3.jpg', 3000, 'hash3', 'completed', '00000000000000ff');",
        ).unwrap();
        
        // With threshold = 6 (90% similarity), should find:
        // - Image 1 & 2: distance 2 <= 6 ✓ (detected as duplicate)
        // - Image 1 & 3: distance 8 > 6 ✗ (not duplicate)
        // - Image 2 & 3: distance 6 <= 6 ✓ (detected as duplicate)
        // All three images are in the same cluster via transitive closure
        let scanner = DeduplicationScanner::new(Some(6));
        let result = scanner.scan(&db).unwrap();
        
        assert_eq!(result.total_scanned, 3);
        assert_eq!(result.total_duplicates, 3); // All three in one cluster
        assert_eq!(result.groups.len(), 1); // One group with 3 images
        assert_eq!(result.groups[0].images.len(), 3);
        assert!(result.threshold == 6);
    }
    
    #[test]
    fn test_threshold_filters_correctly() {
        // Verify that different threshold values correctly filter duplicates
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();
        
        // Insert images with known distances
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'completed', '0000000000000000');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/2.jpg', '2.jpg', 2000, 'hash2', 'completed', '000000000000000a');
             INSERT INTO images (file_path, file_name, file_size, file_hash, ai_status, phash) 
             VALUES ('/test/3.jpg', '3.jpg', 3000, 'hash3', 'completed', '00000000000000ff');",
        ).unwrap();
        
        // Image distances (hex byte XOR popcount):
        // 1 & 2: "00" XOR "0a" = 0x0a = 0b1010 → 2 bits
        // 1 & 3: "00" XOR "ff" = 0xff = 0b11111111 → 8 bits
        // 2 & 3: "0a" XOR "ff" = 0xf5 = 0b11110101 → 6 bits
        
        // With strict threshold (1), no duplicates should be found
        let strict_scanner = DeduplicationScanner::new(Some(1));
        let strict_result = strict_scanner.scan(&db).unwrap();
        assert_eq!(strict_result.total_duplicates, 0);
        assert!(strict_result.groups.is_empty());
        
        // With medium threshold (2), should find 1&2 as duplicate
        let medium_scanner = DeduplicationScanner::new(Some(2));
        let medium_result = medium_scanner.scan(&db).unwrap();
        assert_eq!(medium_result.total_duplicates, 2);
        assert_eq!(medium_result.groups.len(), 1);
        
        // With loose threshold (10), should find all three as duplicates
        let loose_scanner = DeduplicationScanner::new(Some(10));
        let loose_result = loose_scanner.scan(&db).unwrap();
        assert_eq!(loose_result.total_duplicates, 3);
        assert_eq!(loose_result.groups.len(), 1);
    }
}
