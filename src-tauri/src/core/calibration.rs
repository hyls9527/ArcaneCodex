#![allow(missing_docs)]
use crate::core::db::Database;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

const DEFAULT_N_BINS: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSample {
    pub id: Option<i64>,
    pub image_id: i64,
    pub predicted_category: String,
    pub raw_confidence: f64,
    pub is_correct: bool,
    pub annotated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub id: Option<i64>,
    pub ece: f64,
    pub mce: f64,
    pub total_samples: i64,
    pub n_bins: usize,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationCurvePoint {
    pub bin_index: usize,
    pub confidence_avg: f64,
    pub accuracy: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinStats {
    pub bin_index: usize,
    pub confidence_avg: f64,
    pub accuracy: f64,
    pub sample_count: usize,
    pub correct_count: usize,
}

struct BinData {
    confidences: Vec<f64>,
    correct_count: usize,
}

pub struct CalibrationEngine {
    db: Arc<Database>,
}

impl CalibrationEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn add_sample(
        &self,
        image_id: i64,
        predicted_category: &str,
        raw_confidence: f64,
        is_correct: bool,
    ) -> AppResult<i64> {
        if !(0.0..=1.0).contains(&raw_confidence) {
            return Err(AppError::validation(format!(
                "置信度必须在 [0, 1] 范围内，当前值: {}",
                raw_confidence
            )));
        }

        let conn = self.db.open_connection()?;
        conn.execute(
            "INSERT INTO calibration_samples (image_id, predicted_category, raw_confidence, is_correct, annotated_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            rusqlite::params![image_id, predicted_category, raw_confidence, is_correct],
        )
        .map_err(AppError::database)?;

        let sample_id = conn.last_insert_rowid();
        info!(
            "校准样本已记录: image_id={}, category={}, confidence={:.3}, is_correct={}",
            image_id, predicted_category, raw_confidence, is_correct
        );
        Ok(sample_id)
    }

    pub fn calculate_ece(&self, n_bins: Option<usize>) -> AppResult<(f64, Vec<BinStats>)> {
        let n_bins = n_bins.unwrap_or(DEFAULT_N_BINS);
        let samples = self.load_all_samples()?;
        let total = samples.len();

        if total == 0 {
            return Ok((0.0, vec![]));
        }

        let bins = self.bin_samples(&samples, n_bins);
        let mut ece = 0.0;
        let mut bin_stats_list = Vec::with_capacity(n_bins);

        for (bin_index, bin_data) in bins.iter().enumerate() {
            if bin_data.confidences.is_empty() {
                continue;
            }
            let count = bin_data.confidences.len();
            let confidence_avg: f64 = bin_data.confidences.iter().sum::<f64>() / count as f64;
            let accuracy = bin_data.correct_count as f64 / count as f64;
            let bin_weight = count as f64 / total as f64;
            ece += bin_weight * (accuracy - confidence_avg).abs();

            bin_stats_list.push(BinStats {
                bin_index,
                confidence_avg,
                accuracy,
                sample_count: count,
                correct_count: bin_data.correct_count,
            });
        }

        info!(
            "ECE 计算完成: {:.4} ({} 个样本, {} 个 bin)",
            ece, total, n_bins
        );
        Ok((ece, bin_stats_list))
    }

    pub fn calculate_mce(&self, n_bins: Option<usize>) -> AppResult<(f64, Vec<BinStats>)> {
        let n_bins = n_bins.unwrap_or(DEFAULT_N_BINS);
        let samples = self.load_all_samples()?;
        let total = samples.len();

        if total == 0 {
            return Ok((0.0, vec![]));
        }

        let bins = self.bin_samples(&samples, n_bins);
        let mut mce: f64 = 0.0;
        let mut bin_stats_list = Vec::with_capacity(n_bins);

        for (bin_index, bin_data) in bins.iter().enumerate() {
            if bin_data.confidences.is_empty() {
                continue;
            }
            let count = bin_data.confidences.len();
            let confidence_avg: f64 = bin_data.confidences.iter().sum::<f64>() / count as f64;
            let accuracy = bin_data.correct_count as f64 / count as f64;
            let deviation = (accuracy - confidence_avg).abs();
            mce = mce.max(deviation);

            bin_stats_list.push(BinStats {
                bin_index,
                confidence_avg,
                accuracy,
                sample_count: count,
                correct_count: bin_data.correct_count,
            });
        }

        info!(
            "MCE 计算完成: {:.4} ({} 个样本, {} 个 bin)",
            mce, total, n_bins
        );
        Ok((mce, bin_stats_list))
    }

    pub fn generate_report(&self, n_bins: Option<usize>) -> AppResult<CalibrationReport> {
        let n_bins = n_bins.unwrap_or(DEFAULT_N_BINS);

        let (ece, bin_stats_ece) = self.calculate_ece(Some(n_bins))?;
        let (mce, _bin_stats_mce) = self.calculate_mce(Some(n_bins))?;

        let total_samples = self.count_samples()?;
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let report_json_value = serde_json::json!({
            "ece": ece,
            "mce": mce,
            "n_bins": n_bins,
            "total_samples": total_samples,
            "bins": bin_stats_ece,
            "computed_at": now
        });
        let report_json = serde_json::to_string(&report_json_value)
            .map_err(|e| AppError::ai(format!("序列化校准报告失败: {}", e)))?;

        let conn = self.db.open_connection()?;
        conn.execute(
            "INSERT INTO calibration_reports (report_json, total_samples, overall_ece, computed_at, created_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            rusqlite::params![report_json, total_samples, ece, now],
        )
        .map_err(AppError::database)?;

        let report_id = conn.last_insert_rowid();

        self.save_calibration_curve(&bin_stats_ece, "all", total_samples, &now)?;

        info!(
            "校准报告已生成: id={}, ECE={:.4}, MCE={:.4}, samples={}",
            report_id, ece, mce, total_samples
        );

        Ok(CalibrationReport {
            id: Some(report_id),
            ece,
            mce,
            total_samples,
            n_bins,
            computed_at: now,
        })
    }

    pub fn get_latest_report(&self) -> AppResult<Option<CalibrationReport>> {
        let conn = self.db.open_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, report_json, total_samples, overall_ece, computed_at, created_at
                 FROM calibration_reports ORDER BY id DESC LIMIT 1",
            )
            .map_err(AppError::database)?;

        let result = stmt.query_row([], |row| {
            let report_json: String = row.get(1)?;
            let parsed: serde_json::Value =
                serde_json::from_str(&report_json).unwrap_or(serde_json::Value::Null);
            let mce = parsed["mce"].as_f64().unwrap_or(0.0);
            let n_bins = parsed["n_bins"].as_u64().unwrap_or(10) as usize;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, String>(4)?,
                mce,
                n_bins,
            ))
        });

        match result {
            Ok((id, total_samples, ece, computed_at, mce, n_bins)) => Ok(Some(CalibrationReport {
                id: Some(id),
                ece,
                mce,
                total_samples,
                n_bins,
                computed_at,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::database(e)),
        }
    }

    pub fn get_calibration_curve(&self) -> AppResult<Vec<CalibrationCurvePoint>> {
        let (_ece, bin_stats) = self.calculate_ece(Some(DEFAULT_N_BINS))?;
        let curve_points: Vec<CalibrationCurvePoint> = bin_stats
            .iter()
            .map(|b| CalibrationCurvePoint {
                bin_index: b.bin_index,
                confidence_avg: b.confidence_avg,
                accuracy: b.accuracy,
                sample_count: b.sample_count,
            })
            .collect();
        Ok(curve_points)
    }

    fn load_all_samples(&self) -> AppResult<Vec<CalibrationSample>> {
        let conn = self.db.open_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, image_id, predicted_category, raw_confidence, is_correct, annotated_at
                 FROM calibration_samples",
            )
            .map_err(AppError::database)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CalibrationSample {
                    id: row.get(0)?,
                    image_id: row.get(1)?,
                    predicted_category: row.get(2)?,
                    raw_confidence: row.get(3)?,
                    is_correct: row.get(4)?,
                    annotated_at: row.get(5)?,
                })
            })
            .map_err(AppError::database)?;

        let mut samples = Vec::new();
        for row in rows {
            samples.push(row.map_err(AppError::database)?);
        }
        Ok(samples)
    }

    fn bin_samples(&self, samples: &[CalibrationSample], n_bins: usize) -> Vec<BinData> {
        let mut bins: Vec<BinData> = (0..n_bins)
            .map(|_| BinData {
                confidences: Vec::new(),
                correct_count: 0,
            })
            .collect();

        for sample in samples {
            let bin_idx =
                ((sample.raw_confidence * n_bins as f64).floor() as usize).min(n_bins - 1);
            bins[bin_idx].confidences.push(sample.raw_confidence);
            if sample.is_correct {
                bins[bin_idx].correct_count += 1;
            }
        }
        bins
    }

    pub fn count_samples(&self) -> AppResult<i64> {
        let conn = self.db.open_connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM calibration_samples", [], |row| {
                row.get(0)
            })
            .map_err(AppError::database)?;
        Ok(count)
    }

    fn save_calibration_curve(
        &self,
        bin_stats: &[BinStats],
        category: &str,
        total_samples: i64,
        computed_at: &str,
    ) -> AppResult<()> {
        let curve_json = serde_json::to_string(bin_stats)
            .map_err(|e| AppError::ai(format!("序列化校准曲线失败: {}", e)))?;

        let conn = self.db.open_connection()?;
        conn.execute(
            "INSERT OR REPLACE INTO calibration_curves (category, curve_json, total_samples, computed_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![category, curve_json, total_samples, computed_at],
        )
        .map_err(AppError::database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Database;

    fn setup_test_engine() -> (CalibrationEngine, tempfile::TempDir) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_calibration.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
        (CalibrationEngine::new(std::sync::Arc::new(db)), temp_dir)
    }

    #[test]
    fn test_perfectly_calibrated_model_ece_zero() {
        let (engine, _temp) = setup_test_engine();

        let test_cases = vec![
            (1, "nature", 0.05, false),
            (2, "nature", 0.15, false),
            (3, "portrait", 0.25, false),
            (4, "food", 0.35, false),
            (5, "landscape", 0.45, true),
            (6, "landscape", 0.55, true),
            (7, "portrait", 0.65, true),
            (8, "nature", 0.75, true),
            (9, "food", 0.85, true),
            (10, "landscape", 0.95, true),
        ];

        for (id, cat, conf, correct) in &test_cases {
            engine.add_sample(*id, cat, *conf, *correct).unwrap();
        }

        let (ece, bins) = engine.calculate_ece(Some(10)).unwrap();
        assert!(ece < 0.01, "完美校准模型 ECE 应接近 0，实际值: {:.4}", ece);
        assert_eq!(bins.len(), 10, "应有 10 个 bin");
    }

    #[test]
    fn test_overconfident_model_positive_ece() {
        let (engine, _temp) = setup_test_engine();

        let test_cases = vec![
            (1, "nature", 0.90, false),
            (2, "portrait", 0.85, false),
            (3, "food", 0.80, false),
            (4, "landscape", 0.95, false),
            (5, "nature", 0.88, false),
            (6, "portrait", 0.92, false),
            (7, "food", 0.87, false),
            (8, "landscape", 0.91, false),
            (9, "nature", 0.89, false),
            (10, "portrait", 0.93, true),
        ];

        for (id, cat, conf, correct) in &test_cases {
            engine.add_sample(*id, cat, *conf, *correct).unwrap();
        }

        let (ece, _) = engine.calculate_ece(Some(10)).unwrap();
        assert!(
            ece > 0.3,
            "过度自信模型 ECE 应显著大于 0，实际值: {:.4}",
            ece
        );
    }

    #[test]
    fn test_underconfident_model_positive_ece() {
        let (engine, _temp) = setup_test_engine();

        let test_cases = vec![
            (1, "nature", 0.10, true),
            (2, "portrait", 0.15, true),
            (3, "food", 0.20, true),
            (4, "landscape", 0.05, true),
            (5, "nature", 0.12, true),
            (6, "portrait", 0.08, true),
            (7, "food", 0.18, true),
            (8, "landscape", 0.22, true),
            (9, "nature", 0.09, true),
            (10, "portrait", 0.14, true),
        ];

        for (id, cat, conf, correct) in &test_cases {
            engine.add_sample(*id, cat, *conf, *correct).unwrap();
        }

        let (ece, _) = engine.calculate_ece(Some(10)).unwrap();
        assert!(
            ece > 0.3,
            "不足自信模型 ECE 应显著大于 0，实际值: {:.4}",
            ece
        );
    }

    #[test]
    fn test_mce_equals_max_bin_deviation() {
        let (engine, _temp) = setup_test_engine();

        for i in 0..20i64 {
            let conf = if i < 10 { 0.95 } else { 0.05 };
            let correct = i < 10;
            engine.add_sample(i, "test", conf, correct).unwrap();
        }

        let (mce, bins) = engine.calculate_mce(Some(10)).unwrap();
        let max_deviation = bins
            .iter()
            .filter(|b| b.sample_count > 0)
            .map(|b| (b.accuracy - b.confidence_avg).abs())
            .fold(0.0_f64, f64::max);

        assert!(
            (mce - max_deviation).abs() < 1e-10,
            "MCE 应等于最大 bin 偏差"
        );
    }

    #[test]
    fn test_empty_samples_returns_zero() {
        let (engine, _temp) = setup_test_engine();

        let (ece, bins) = engine.calculate_ece(Some(10)).unwrap();
        assert_eq!(ece, 0.0, "无样本时 ECE 应为 0");
        assert!(bins.is_empty(), "无样本时 bins 应为空");

        let (mce, mce_bins) = engine.calculate_mce(Some(10)).unwrap();
        assert_eq!(mce, 0.0, "无样本时 MCE 应为 0");
        assert!(mce_bins.is_empty(), "无样本时 MCE bins 应为空");
    }

    #[test]
    fn test_add_sample_validates_confidence_range() {
        let (engine, _temp) = setup_test_engine();

        let result = engine.add_sample(1, "test", 1.5, true);
        assert!(result.is_err(), "置信度 > 1.0 应报错");

        let result = engine.add_sample(1, "test", -0.1, true);
        assert!(result.is_err(), "置信度 < 0.0 应报错");

        let result = engine.add_sample(1, "test", 0.0, true);
        assert!(result.is_ok(), "置信度 0.0 应合法");

        let result = engine.add_sample(2, "test", 1.0, true);
        assert!(result.is_ok(), "置信度 1.0 应合法");
    }

    #[test]
    fn test_generate_report_persists_to_db() {
        let (engine, _temp) = setup_test_engine();

        for i in 0..20i64 {
            let conf = (i as f64) / 19.0;
            let correct = i % 3 != 0;
            engine.add_sample(i, "nature", conf, correct).unwrap();
        }

        let report = engine.generate_report(Some(10)).unwrap();
        assert!(report.id.unwrap() > 0, "报告 ID 应大于 0");
        assert!(report.ece >= 0.0, "ECE 应非负");
        assert!(report.mce >= 0.0, "MCE 应非负");
        assert_eq!(report.total_samples, 20, "总样本数应为 20");
        assert_eq!(report.n_bins, 10, "Bin 数应为 10");
        assert!(!report.computed_at.is_empty(), "计算时间不应为空");
    }

    #[test]
    fn test_get_latest_report_returns_most_recent() {
        let (engine, _temp) = setup_test_engine();

        engine.add_sample(1, "cat", 0.9, true).unwrap();
        let report1 = engine.generate_report(Some(10)).unwrap();

        engine.add_sample(2, "cat", 0.3, false).unwrap();
        let report2 = engine.generate_report(Some(10)).unwrap();

        let latest = engine.get_latest_report().unwrap().unwrap();
        assert_eq!(latest.id, report2.id, "最新报告应为第二次生成的");
        assert!(
            latest.id.unwrap() > report1.id.unwrap(),
            "最新报告 ID 应更大"
        );
    }

    #[test]
    fn test_get_latest_report_empty_returns_none() {
        let (engine, _temp) = setup_test_engine();
        let result = engine.get_latest_report().unwrap();
        assert!(result.is_none(), "无报告时应返回 None");
    }

    #[test]
    fn test_get_calibration_curve_structure() {
        let (engine, _temp) = setup_test_engine();

        for i in 0..50i64 {
            let conf = (i as f64) / 49.0;
            let correct = conf >= 0.5;
            engine.add_sample(i, "test", conf, correct).unwrap();
        }

        let curve = engine.get_calibration_curve().unwrap();
        let non_empty_bins: Vec<_> = curve.iter().filter(|p| p.sample_count > 0).collect();

        assert!(!non_empty_bins.is_empty(), "校准曲线应有非空 bin");

        for point in &curve {
            assert!(
                (0.0..=1.0).contains(&point.confidence_avg),
                "平均置信度应在 [0,1] 范围内"
            );
            assert!(
                (0.0..=1.0).contains(&point.accuracy),
                "准确率应在 [0,1] 范围内"
            );
        }
    }

    #[test]
    fn test_custom_bin_count() {
        let (engine, _temp) = setup_test_engine();

        for i in 0..100i64 {
            let conf = (i as f64) / 99.0;
            let correct = conf >= 0.5;
            engine.add_sample(i, "test", conf, correct).unwrap();
        }

        let (_ece_5, bins_5) = engine.calculate_ece(Some(5)).unwrap();
        let (_ece_20, bins_20) = engine.calculate_ece(Some(20)).unwrap();

        assert_eq!(bins_5.len(), 5, "自定义 5 个 bin");
        assert_eq!(bins_20.len(), 20, "自定义 20 个 bin");
    }

    #[test]
    fn test_boundary_confidence_values() {
        let (engine, _temp) = setup_test_engine();

        engine.add_sample(1, "test", 0.0, false).unwrap();
        engine.add_sample(2, "test", 1.0, true).unwrap();
        engine.add_sample(3, "test", 0.999999, true).unwrap();
        engine.add_sample(4, "test", 0.000001, false).unwrap();

        let (ece, _) = engine.calculate_ece(Some(10)).unwrap();
        assert!(ece.is_finite(), "边界置信度值不应产生 NaN 或 Inf");
    }
}
