use std::fs;
use std::time::Instant;

fn main() {
    println!("=== ArcaneGallery v2 ONNX模型推理速度验证 ===\n");

    verify_mobilenetv2();
    verify_insightface_det();
    verify_insightface_rec();

    println!("\n=== 验证完成 ===");
}

fn run_inference_test(model_path: &str, input_dims: (usize, usize, usize, usize), label: &str, target_ms: u64) {
    if !std::path::Path::new(model_path).exists() {
        println!("  [SKIP] 模型文件不存在: {}", model_path);
        return;
    }

    let file_size = fs::metadata(model_path).unwrap().len();
    println!("  [INFO] 模型大小: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);

    let start = Instant::now();
    let mut session = match ort::session::Session::builder()
        .and_then(|mut b| b.commit_from_file(model_path))
    {
        Ok(s) => s,
        Err(e) => {
            println!("  [FAIL] 模型加载失败: {}", e);
            return;
        }
    };
    let load_time = start.elapsed();
    println!("  [PASS] 模型加载成功, 耗时: {:?}", load_time);

    let (n, c, h, w) = input_dims;
    let total = n * c * h * w;
    let input_data: Vec<f32> = vec![0.5f32; total];

    let input_value = match ort::value::Value::from_array((
        vec![n, c, h, w],
        input_data.into_boxed_slice()
    )) {
        Ok(v) => v,
        Err(e) => {
            println!("  [FAIL] 创建输入Value失败: {}", e);
            return;
        }
    };

    let mut times = Vec::new();
    for i in 0..5 {
        let start = Instant::now();
        match session.run(ort::inputs![input_value.clone()]) {
            Ok(_outputs) => {
                let elapsed = start.elapsed();
                times.push(elapsed);
            }
            Err(e) => {
                println!("  [FAIL] 推理失败(第{}次): {}", i + 1, e);
                return;
            }
        }
    }

    println!("\n  推理速度结果:");
    for (i, t) in times.iter().enumerate() {
        println!("    第{}次: {:?}", i + 1, t);
    }
    let avg: std::time::Duration = times[1..].iter().sum::<std::time::Duration>() / (times.len() - 1) as u32;
    println!("    平均(排除首次): {:?}", avg);

    if avg < std::time::Duration::from_millis(target_ms) {
        println!("  [PASS] ✅ {} 推理速度 < {}ms，满足要求！", label, target_ms);
    } else {
        println!("  [WARN] ⚠️ {} 推理速度 > {}ms，需优化", label, target_ms);
    }
}

fn verify_mobilenetv2() {
    println!("--- 验证4: MobileNetV2 ONNX 推理速度 ---");
    run_inference_test("models/mobilenetv2.onnx", (1, 3, 224, 224), "MobileNetV2", 100);
}

fn verify_insightface_det() {
    println!("\n--- 验证8a: InsightFace 人脸检测 (det_500m.onnx) ---");
    run_inference_test("models/buffalo_sc/det_500m.onnx", (1, 3, 640, 640), "人脸检测", 200);
}

fn verify_insightface_rec() {
    println!("\n--- 验证8b: InsightFace 人脸识别 (w600k_mbf.onnx) ---");
    run_inference_test("models/buffalo_sc/w600k_mbf.onnx", (1, 3, 112, 112), "人脸识别", 200);
}
