use crate::utils::error::{AppError, AppResult};
use image::GenericImageView;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::info;

const MAX_THUMBNAIL_WIDTH: u32 = 300;
const MAX_THUMBNAIL_HEIGHT: u32 = 200;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn generate_thumbnail(image_path: &str, output_path: &str) -> AppResult<()> {
        let img_path = Path::new(image_path);
        let out_path = Path::new(output_path);

        if !img_path.exists() {
            return Err(AppError::validation(format!(
                "源图片不存在: {}", image_path
            )));
        }

        let img = image::open(img_path).map_err(|e| {
            AppError::validation(format!("无法打开图片 {}: {}", image_path, e))
        })?;

        let thumbnail = img.thumbnail(MAX_THUMBNAIL_WIDTH, MAX_THUMBNAIL_HEIGHT);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        }

        thumbnail.save_with_format(out_path, image::ImageFormat::WebP).map_err(|e| {
            AppError::validation(format!("保存缩略图失败 {}: {}", output_path, e))
        })?;

        info!("缩略图生成成功: {} -> {}", image_path, output_path);

        Ok(())
    }

    pub fn calculate_phash(image_path: &str) -> AppResult<String> {
        let img_path = Path::new(image_path);

        if !img_path.exists() {
            return Err(AppError::validation(format!(
                "源图片不存在: {}", image_path
            )));
        }

        let img = image::open(img_path).map_err(|e| {
            AppError::validation(format!("无法打开图片 {}: {}", image_path, e))
        })?;

        let rgba = img.to_rgba8();

        let img_resized = image::imageops::resize(
            &rgba,
            32,
            32,
            image::imageops::FilterType::Lanczos3,
        );

        let gray: image::GrayImage = image::imageops::colorops::grayscale(&img_resized);

        let hash_bytes = compute_phash_from_grayscale(&gray);

        Ok(hex::encode(hash_bytes))
    }

    #[allow(dead_code)]
    pub fn hamming_distance(hash1: &str, hash2: &str) -> AppResult<u32> {
        let bytes1 = hex::decode(hash1).map_err(|e| {
            AppError::validation(format!("解析 hash1 失败: {:?}", e))
        })?;

        let bytes2 = hex::decode(hash2).map_err(|e| {
            AppError::validation(format!("解析 hash2 失败: {:?}", e))
        })?;

        if bytes1.len() != bytes2.len() {
            return Err(AppError::validation(format!(
                "哈希长度不匹配: {} vs {}", bytes1.len(), bytes2.len()
            )));
        }

        let mut distance = 0u32;
        for (b1, b2) in bytes1.iter().zip(bytes2.iter()) {
            distance += (b1 ^ b2).count_ones();
        }

        Ok(distance)
    }

    pub fn extract_exif(image_path: &str) -> AppResult<serde_json::Value> {
        let img_path = Path::new(image_path);

        if !img_path.exists() {
            return Err(AppError::validation(format!(
                "源图片不存在: {}", image_path
            )));
        }

        let img = image::open(img_path).map_err(|e| {
            AppError::validation(format!("无法打开图片 {}: {}", image_path, e))
        })?;

        let dimensions = img.dimensions();

        let mut exif_data = serde_json::json!({
            "width": dimensions.0,
            "height": dimensions.1,
            "has_exif": false
        });

        let file = File::open(img_path).map_err(|e| {
            AppError::validation(format!("无法打开文件: {}", e))
        })?;

        let mut bufreader = BufReader::new(&file);
        let exifreader = exif::Reader::new();
        
        match exifreader.read_from_container(&mut bufreader) {
            Ok(exif) => {
                let mut exif_json = serde_json::Map::new();
                
                exif_json.insert("has_exif".to_string(), serde_json::json!(true));
                exif_json.insert("width".to_string(), serde_json::json!(dimensions.0));
                exif_json.insert("height".to_string(), serde_json::json!(dimensions.1));

                if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
                    let datetime_str = field.display_value().to_string();
                    exif_json.insert("datetime_original".to_string(), serde_json::json!(datetime_str));
                }

                if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
                    let datetime_str = field.display_value().to_string();
                    exif_json.insert("datetime".to_string(), serde_json::json!(datetime_str));
                }

                if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
                    let make = field.display_value().to_string();
                    exif_json.insert("make".to_string(), serde_json::json!(make));
                }

                if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
                    let model = field.display_value().to_string();
                    exif_json.insert("model".to_string(), serde_json::json!(model));
                }

                if let Some(field) = exif.get_field(exif::Tag::Software, exif::In::PRIMARY) {
                    let software = field.display_value().to_string();
                    exif_json.insert("software".to_string(), serde_json::json!(software));
                }

                if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
                    let fnumber = field.display_value().to_string();
                    exif_json.insert("fnumber".to_string(), serde_json::json!(fnumber));
                }

                if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
                    let exposure = field.display_value().to_string();
                    exif_json.insert("exposure_time".to_string(), serde_json::json!(exposure));
                }

                if let Some(field) = exif.get_field(exif::Tag::ISOSpeed, exif::In::PRIMARY) {
                    let iso = field.display_value().to_string();
                    exif_json.insert("iso".to_string(), serde_json::json!(iso));
                }

                if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
                    let focal = field.display_value().to_string();
                    exif_json.insert("focal_length".to_string(), serde_json::json!(focal));
                }

                if let Some(field) = exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY) {
                    let lat = field.display_value().to_string();
                    exif_json.insert("gps_latitude".to_string(), serde_json::json!(lat));
                }

                if let Some(field) = exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY) {
                    let lon = field.display_value().to_string();
                    exif_json.insert("gps_longitude".to_string(), serde_json::json!(lon));
                }

                exif_data = serde_json::Value::Object(exif_json);
                
                info!("EXIF 提取成功: {}", image_path);
            }
            Err(_) => {
                info!("图片不包含 EXIF 数据或提取失败: {}", image_path);
            }
        }

        Ok(exif_data)
    }
}

fn compute_phash_from_grayscale(img: &image::GrayImage) -> Vec<u8> {
    let (width, height) = img.dimensions();
    let pixels: Vec<f64> = img.pixels().map(|p| p[0] as f64).collect();

    let n = (width * height) as usize;
    if n == 0 {
        return vec![0; 8];
    }

    let mean: f64 = pixels.iter().sum::<f64>() / n as f64;

    let mut hash_bits = Vec::with_capacity(n / 8);
    let mut current_byte: u8 = 0;
    let mut bit_count = 0;

    for &p in &pixels {
        if p >= mean {
            current_byte |= 1 << (7 - bit_count);
        }
        bit_count += 1;

        if bit_count == 8 {
            hash_bits.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }

    if bit_count > 0 {
        hash_bits.push(current_byte);
    }

    hash_bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_image(path: &Path, width: u32, height: u32) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        if ext == "jpg" || ext == "jpeg" {
            let img = image::RgbImage::from_pixel(width, height, image::Rgb([255, 0, 0]));
            img.save(path).unwrap();
        } else {
            let img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
            img.save(path).unwrap();
        }
    }

    #[test]
    fn test_generate_thumbnail_jpeg() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_thumb.jpg");
        let out_path = temp_dir.join("thumb_output.webp");

        create_test_image(&src_path, 800, 600);

        let result = ImageProcessor::generate_thumbnail(
            src_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        );
        assert!(result.is_ok(), "缩略图生成应成功: {:?}", result);

        assert!(out_path.exists(), "输出文件应存在");

        let metadata = fs::metadata(&out_path).unwrap();
        assert!(metadata.len() > 0, "输出文件不应为空");

        let thumb = image::open(&out_path).unwrap();
        let (w, h) = thumb.dimensions();
        assert!(w <= MAX_THUMBNAIL_WIDTH, "宽度不应超过 {}", MAX_THUMBNAIL_WIDTH);
        assert!(h <= MAX_THUMBNAIL_HEIGHT, "高度不应超过 {}", MAX_THUMBNAIL_HEIGHT);

        fs::remove_file(&src_path).ok();
        fs::remove_file(&out_path).ok();
    }

    #[test]
    fn test_generate_thumbnail_png() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_png");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_thumb.png");
        let out_path = temp_dir.join("thumb_output.png.webp");

        create_test_image(&src_path, 1920, 1080);

        let result = ImageProcessor::generate_thumbnail(
            src_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        );
        assert!(result.is_ok());

        let thumb = image::open(&out_path).unwrap();
        let (w, h) = thumb.dimensions();
        assert!(w <= MAX_THUMBNAIL_WIDTH);
        assert!(h <= MAX_THUMBNAIL_HEIGHT);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_small_image_no_resize() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_small");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_small.jpg");
        let out_path = temp_dir.join("thumb_small.webp");

        create_test_image(&src_path, 100, 80);

        let result = ImageProcessor::generate_thumbnail(
            src_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        );
        assert!(result.is_ok());

        let thumb = image::open(&out_path).unwrap();
        let (w, h) = thumb.dimensions();
        // 缩略图不应超过最大尺寸
        assert!(w <= MAX_THUMBNAIL_WIDTH, "宽度不应超过 {}", MAX_THUMBNAIL_WIDTH);
        assert!(h <= MAX_THUMBNAIL_HEIGHT, "高度不应超过 {}", MAX_THUMBNAIL_HEIGHT);
        // 保持宽高比
        assert_eq!((w as f32 / h as f32 * 100.0) as u32, 125, "应保持宽高比");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_nonexistent_file() {
        let result = ImageProcessor::generate_thumbnail(
            "/nonexistent/path/image.jpg",
            "/tmp/thumb.webp",
        );
        assert!(result.is_err(), "不存在的文件应返回错误");
    }

    #[test]
    fn test_generate_thumbnail_creates_output_dir() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_nested");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_nested.jpg");
        let nested_dir = temp_dir.join("nested").join("output");
        let out_path = nested_dir.join("thumb.webp");

        create_test_image(&src_path, 500, 400);

        assert!(!nested_dir.exists(), "嵌套目录不应预先存在");

        let result = ImageProcessor::generate_thumbnail(
            src_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(out_path.exists(), "输出文件应在嵌套目录中创建");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_calculate_phash() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_phash");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_phash.png");
        create_test_image(&src_path, 256, 256);

        let result = ImageProcessor::calculate_phash(src_path.to_str().unwrap());
        assert!(result.is_ok(), "pHash 计算应成功: {:?}", result);

        let hash = result.unwrap();
        assert!(!hash.is_empty(), "pHash 不应为空");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_hamming_distance_same_image() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_hamming");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_hamming.png");
        create_test_image(&src_path, 256, 256);

        let hash = ImageProcessor::calculate_phash(src_path.to_str().unwrap()).unwrap();
        let result = ImageProcessor::hamming_distance(&hash, &hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0, "相同图片的 Hamming 距离应为 0");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_extract_exif() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_exif");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_exif.jpg");
        create_test_image(&src_path, 640, 480);

        let result = ImageProcessor::extract_exif(src_path.to_str().unwrap());
        assert!(result.is_ok());

        let exif = result.unwrap();
        assert!(exif.get("width").is_some());
        assert!(exif.get("height").is_some());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_extract_exif_nonexistent_file() {
        let result = ImageProcessor::extract_exif("/nonexistent/path/image.jpg");
        assert!(result.is_err(), "不存在的文件应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("源图片不存在"));
    }

    #[test]
    fn test_extract_exif_no_exif_data() {
        let temp_dir = std::env::temp_dir().join("arcanecodex_test_no_exif");
        fs::create_dir_all(&temp_dir).unwrap();

        let src_path = temp_dir.join("test_no_exif.png");
        create_test_image(&src_path, 800, 600);

        let result = ImageProcessor::extract_exif(src_path.to_str().unwrap());
        assert!(result.is_ok(), "无 EXIF 数据的图片也应成功返回");

        let exif = result.unwrap();
        assert!(exif.get("width").is_some());
        assert!(exif.get("height").is_some());

        fs::remove_dir_all(&temp_dir).ok();
    }
}
