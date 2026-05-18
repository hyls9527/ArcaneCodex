// Integration tests for image_classifier module
// Tests cover: ClassificationError Display/From, struct construction,
// ImageClassifier creation, classify_image error path, OnnxRuntimeManager initial state

#[cfg(test)]
mod tests {
    use arcane_codex::core::image_classifier::*;
    use arcane_codex::core::onnx_runtime::{ModelType, OnnxRuntimeManager};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_classification_error_display_model_not_loaded() {
        let err = ClassificationError::ModelNotLoaded;
        assert_eq!(format!("{}", err), "图像分类模型未加载");
    }

    #[test]
    fn test_classification_error_display_image_processing_failed() {
        let err = ClassificationError::ImageProcessingFailed("bad image".to_string());
        assert_eq!(format!("{}", err), "图像处理失败: bad image");
    }

    #[test]
    fn test_classification_error_display_inference_error() {
        let err = ClassificationError::InferenceError("timeout".to_string());
        assert_eq!(format!("{}", err), "推理错误: timeout");
    }

    #[test]
    fn test_classification_error_from_image_error() {
        // Create an ImageError by loading invalid image data
        let bad_data = vec![0u8; 10];
        let result = image::load_from_memory(&bad_data);
        assert!(result.is_err());
        let img_err = result.unwrap_err();
        let class_err: ClassificationError = img_err.into();
        assert!(matches!(class_err, ClassificationError::ImageProcessingFailed(_)));
    }

    #[test]
    fn test_image_classification_result_construction() {
        let predictions = vec![
            ClassPrediction {
                class_name: "cat".to_string(),
                confidence: 0.95,
            },
            ClassPrediction {
                class_name: "dog".to_string(),
                confidence: 0.85,
            },
        ];
        let result = ImageClassificationResult {
            label: "cat".to_string(),
            confidence: 0.95,
            top_n_predictions: predictions,
        };
        assert_eq!(result.label, "cat");
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.top_n_predictions.len(), 2);

        // Test with empty predictions
        let empty_result = ImageClassificationResult {
            label: "unknown".to_string(),
            confidence: 0.0,
            top_n_predictions: vec![],
        };
        assert!(empty_result.top_n_predictions.is_empty());
    }

    #[test]
    fn test_class_prediction_construction() {
        let pred = ClassPrediction {
            class_name: "goldfish".to_string(),
            confidence: 0.92,
        };
        assert_eq!(pred.class_name, "goldfish");
        assert_eq!(pred.confidence, 0.92);
    }

    #[test]
    fn test_image_classifier_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = Arc::new(OnnxRuntimeManager::new(temp_dir.path()));
        let _classifier = ImageClassifier::new(manager);
    }

    #[tokio::test]
    async fn test_classify_image_model_not_loaded() {
        let temp_dir = TempDir::new().unwrap();
        let manager = Arc::new(OnnxRuntimeManager::new(temp_dir.path()));
        let classifier = ImageClassifier::new(manager);
        let result = classifier
            .classify_image(std::path::Path::new("dummy.jpg"), 5)
            .await;
        assert!(matches!(result, Err(ClassificationError::ModelNotLoaded)));
    }

    #[tokio::test]
    async fn test_onnx_runtime_manager_initial_state() {
        let temp_dir = TempDir::new().unwrap();
        let manager = OnnxRuntimeManager::new(temp_dir.path());

        let loaded = manager
            .is_model_loaded(ModelType::ImageClassification)
            .await;
        assert!(!loaded, "Model should not be loaded initially");

        let unloaded = manager
            .unload_model(ModelType::ImageClassification)
            .await;
        assert!(!unloaded, "Unloading non-loaded model should return false");

        let status = manager.get_model_status().await;
        assert!(status.is_empty(), "Model status should be empty initially");
    }

    #[test]
    fn test_imagenet_classes_non_empty() {
        assert!(!IMAGENET_CLASSES.is_empty());
        assert!(IMAGENET_CLASSES.len() >= 1000);
    }
}
