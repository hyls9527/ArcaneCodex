use crate::core::calibration::{CalibrationCurvePoint, CalibrationEngine, CalibrationReport};
use crate::core::db::Database;
use crate::utils::error::AppResult;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct RecordSampleResponse {
    pub success: bool,
    pub sample_id: i64,
    pub message: String,
}

#[tauri::command]
pub fn record_calibration_sample(
    db: State<'_, Arc<Database>>,
    image_id: i64,
    predicted_category: String,
    raw_confidence: f64,
    is_correct: bool,
) -> AppResult<RecordSampleResponse> {
    let engine = CalibrationEngine::new(db.inner().clone());
    let sample_id = engine.add_sample(image_id, &predicted_category, raw_confidence, is_correct)?;

    Ok(RecordSampleResponse {
        success: true,
        sample_id,
        message: format!(
            "校准样本已记录: image_id={}, confidence={:.3}",
            image_id, raw_confidence
        ),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationReportResponse {
    pub success: bool,
    pub report: Option<CalibrationReport>,
    pub message: String,
}

#[tauri::command]
pub fn calculate_and_save_calibration(
    db: State<'_, Arc<Database>>,
    n_bins: Option<usize>,
) -> AppResult<CalibrationReportResponse> {
    let engine = CalibrationEngine::new(db.inner().clone());
    let total_samples = engine.count_samples()?;

    if total_samples == 0 {
        return Ok(CalibrationReportResponse {
            success: false,
            report: None,
            message: "没有校准样本数据，无法计算 ECE".to_string(),
        });
    }

    let report = engine.generate_report(n_bins)?;
    let ece = report.ece;
    let mce = report.mce;

    Ok(CalibrationReportResponse {
        success: true,
        report: Some(report),
        message: format!(
            "校准报告已生成: ECE={:.4}, MCE={:.4}, 样本数={}",
            ece, mce, total_samples
        ),
    })
}

#[tauri::command]
pub fn get_latest_calibration_report(
    db: State<'_, Arc<Database>>,
) -> AppResult<CalibrationReportResponse> {
    let engine = CalibrationEngine::new(db.inner().clone());
    let report = engine.get_latest_report()?;

    match report {
        Some(r) => Ok(CalibrationReportResponse {
            success: true,
            report: Some(r),
            message: "获取最新校准报告成功".to_string(),
        }),
        None => Ok(CalibrationReportResponse {
            success: false,
            report: None,
            message: "暂无校准报告，请先执行校准计算".to_string(),
        }),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationCurveResponse {
    pub success: bool,
    pub curve_data: Vec<CalibrationCurvePoint>,
    pub total_points: usize,
    pub message: String,
}

#[tauri::command]
pub fn get_calibration_curve_data(
    db: State<'_, Arc<Database>>,
) -> AppResult<CalibrationCurveResponse> {
    let engine = CalibrationEngine::new(db.inner().clone());
    let curve = engine.get_calibration_curve()?;
    let total_points = curve.len();

    if total_points == 0 {
        return Ok(CalibrationCurveResponse {
            success: false,
            curve_data: vec![],
            total_points: 0,
            message: "没有校准曲线数据（可能缺少校准样本）".to_string(),
        });
    }

    Ok(CalibrationCurveResponse {
        success: true,
        curve_data: curve.clone(),
        total_points,
        message: format!("校准曲线数据包含 {} 个数据点", total_points),
    })
}
