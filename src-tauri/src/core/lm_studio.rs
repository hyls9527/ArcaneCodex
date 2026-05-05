use crate::core::inference::AIResult;
use crate::utils::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:1234";
const DEFAULT_MODEL: &str = "Qwen2.5-VL-7B-Instruct";
const REQUEST_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

impl Default for LMStudioConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            timeout_secs: REQUEST_TIMEOUT_SECS,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LMStudioModelInfo {
    pub id: String,
    pub owned_by: String,
}

#[derive(Clone)]
pub struct LMStudioClient {
    client: Client,
    pub config: LMStudioConfig,
}

impl LMStudioClient {
    pub fn new(config: LMStudioConfig) -> AppResult<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| AppError::validation(format!("创建 HTTP 客户端失败: {}", e)))?;

        info!("LM Studio 客户端初始化完成: {}", config.base_url);

        Ok(Self { client, config })
    }

    pub async fn health_check(&self) -> AppResult<Vec<LMStudioModelInfo>> {
        let url = format!("{}/v1/models", self.config.base_url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::validation(format!("健康检查请求失败: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::validation(format!(
                "健康检查失败: HTTP {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::validation(format!("解析健康检查响应失败: {}", e)))?;

        let models: Vec<LMStudioModelInfo> = body
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        Some(LMStudioModelInfo {
                            id: m.get("id")?.as_str()?.to_string(),
                            owned_by: m.get("owned_by")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        info!("健康检查成功，可用模型数: {}", models.len());

        Ok(models)
    }

    pub async fn check_model_vision_capability(&self) -> AppResult<bool> {
        let url = format!("{}/v1/models", self.config.base_url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::validation(format!("查询模型能力失败: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::validation(format!(
                "查询模型能力失败: HTTP {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::validation(format!("解析模型列表失败: {}", e)))?;

        let model_id = &self.config.model;

        let model_info = body
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| {
                arr.iter().find(|m| {
                    m.get("id")
                        .and_then(|id| id.as_str())
                        .map(|id| id == model_id)
                        .unwrap_or(false)
                })
            });

        let is_vision = match model_info {
            Some(info) => {
                let has_vision = info
                    .get("capabilities")
                    .and_then(|c| c.as_object())
                    .and_then(|c| c.get("vision"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let id_hints = model_id.to_lowercase();
                let name_hints = info
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let id_vision_hint = id_hints.contains("vl")
                    || id_hints.contains("vision")
                    || id_hints.contains("multimodal")
                    || id_hints.contains("qwen2.5-vl")
                    || id_hints.contains("gemma-4")
                    || id_hints.contains("llava")
                    || id_hints.contains("bakllava")
                    || id_hints.contains("moondream");
                let name_vision_hint = name_hints.contains("vl")
                    || name_hints.contains("vision")
                    || name_hints.contains("multimodal");

                has_vision || id_vision_hint || name_vision_hint
            }
            None => {
                let id = model_id.to_lowercase();
                id.contains("vl")
                    || id.contains("vision")
                    || id.contains("multimodal")
                    || id.contains("qwen2.5-vl")
                    || id.contains("gemma-4")
                    || id.contains("llava")
                    || id.contains("bakllava")
                    || id.contains("moondream")
                    || id.contains("internvl")
                    || id.contains("phi-4")
            }
        };

        info!(
            "模型 {} 视觉能力检测结果: {}",
            model_id,
            if is_vision { "✅ 支持视觉" } else { "❌ 不支持视觉" }
        );

        Ok(is_vision)
    }

    pub async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult> {
        match self.check_model_vision_capability().await {
            Ok(false) => {
                return Err(AppError::validation(format!(
                    "当前模型 '{}' 不支持视觉分析。\n\n💡 解决方案：\n1. 在 LM Studio 中加载一个支持视觉的模型（如 Qwen2.5-VL、LLaVA、Gemma-4 等）\n2. 前往设置 → AI 配置 → 选择正确的模型名称\n3. 模型名称中通常包含 'VL'、'Vision' 或 'Multimodal' 字样\n\n常见视觉模型：Qwen2.5-VL-7B-Instruct、llava-v1.6-mistral-7b、gemma-4-26B-it",
                    self.config.model
                )));
            }
            Ok(true) => {}
            Err(e) => {
                warn!(
                    "无法检测模型 '{}' 的视觉能力（{}），继续尝试推理",
                    self.config.model,
                    e
                );
            }
        }

        let image_base64 = encode_image_to_base64(image_path)?;
        let mime_type = detect_mime_type(image_path)?;
        let prompt = build_prompt();

        let url = format!("{}/v1/chat/completions", self.config.base_url);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:{};base64,{}", mime_type, image_base64)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 500,
            "temperature": 0.1,
            "response_format": { "type": "json_object" }
        });

        let resp = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::validation(format!("AI 推理请求失败: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::validation(format!(
                "AI 推理失败: HTTP {} - {}",
                status, body
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::validation(format!("解析 AI 响应失败: {}", e)))?;

        let content = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AppError::validation("AI 响应格式不正确".to_string()))?;

        let result = parse_ai_response(content, "lm_studio", &self.config.model)?;

        info!(
            "图片分析完成: {} (置信度: {:.2})",
            image_path, result.confidence
        );

        Ok(result)
    }
}

pub fn encode_image_to_base64(image_path: &str) -> AppResult<String> {
    use std::fs;

    let bytes = fs::read(image_path)
        .map_err(|e| AppError::validation(format!("读取图片文件失败: {}", e)))?;

    Ok(data_encoding::BASE64.encode(&bytes))
}

pub fn detect_mime_type(image_path: &str) -> AppResult<String> {
    let path = std::path::Path::new(image_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/jpeg",
    };

    Ok(mime.to_string())
}

pub fn build_prompt() -> String {
    r#"请分析这张图片,并以以下 JSON 格式返回:
{
  "tags": ["标签1", "标签2", "标签3"],
  "description": "一句话描述图片内容",
  "category": "风景|人物|物品|动物|建筑|文档|其他",
  "confidence": 0.95
}
要求:
- tags: 5-10个关键词,中文优先,避免重复和过于宽泛的词
- description: 简洁准确,1-2句话,不超过50字
- category: 从上述分类中选择一个
- confidence: 0.0-1.0之间的数字,表示你的置信度

仅返回合法 JSON,不要包含 Markdown 代码块标记或其他解释。"#
        .to_string()
}

pub fn parse_ai_response(content: &str, provider: &str, model: &str) -> AppResult<crate::core::inference::AIResult> {
    let content = content.trim();
    let content = strip_markdown_wrapper(content);

    let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        AppError::validation(format!(
            "解析 AI JSON 响应失败: {} - 原始内容: {}",
            e, content
        ))
    })?;

    let tags = parsed
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let description = parsed
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("No description")
        .to_string();

    let category = parsed
        .get("category")
        .and_then(|c| c.as_str())
        .unwrap_or("other")
        .to_string();

    let confidence = parsed
        .get("confidence")
        .and_then(|c| c.as_f64())
        .unwrap_or(0.5);

    Ok(crate::core::inference::AIResult {
        tags,
        description,
        category,
        confidence,
        raw_response: content,
        provider: provider.to_string(),
        model: model.to_string(),
    })
}

pub fn strip_markdown_wrapper(content: &str) -> String {
    if content.starts_with("```") {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 2 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            content
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .to_string()
        }
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LMStudioConfig::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.timeout_secs, REQUEST_TIMEOUT_SECS);
    }

    #[test]
    fn test_client_creation() {
        let config = LMStudioConfig::default();
        let client = LMStudioClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_custom_config() {
        let config = LMStudioConfig {
            base_url: "http://localhost:9999".to_string(),
            model: "custom-model".to_string(),
            timeout_secs: 30,
        };
        let client = LMStudioClient::new(config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().config.base_url, "http://localhost:9999");
    }

    #[test]
    fn test_detect_mime_type_jpeg() {
        let mime = detect_mime_type("test.jpg").unwrap();
        assert_eq!(mime, "image/jpeg");

        let mime = detect_mime_type("test.jpeg").unwrap();
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn test_detect_mime_type_png() {
        let mime = detect_mime_type("test.png").unwrap();
        assert_eq!(mime, "image/png");
    }

    #[test]
    fn test_detect_mime_type_webp() {
        let mime = detect_mime_type("test.webp").unwrap();
        assert_eq!(mime, "image/webp");
    }

    #[test]
    fn test_detect_mime_type_unknown() {
        let mime = detect_mime_type("test.unknown").unwrap();
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn test_build_prompt_contains_required_fields() {
        let prompt = build_prompt();
        assert!(prompt.contains("tags"));
        assert!(prompt.contains("description"));
        assert!(prompt.contains("category"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_build_prompt_prd_compliant() {
        let prompt = build_prompt();

        assert!(prompt.contains("请分析这张图片"));
        assert!(prompt.contains("风景"));
        assert!(prompt.contains("人物"));
        assert!(prompt.contains("物品"));
        assert!(prompt.contains("动物"));
        assert!(prompt.contains("建筑"));
        assert!(prompt.contains("文档"));
        assert!(prompt.contains("其他"));
        assert!(prompt.contains("中文优先"));
        assert!(prompt.contains("5-10个关键词"));
        assert!(prompt.contains("1-2句话"));
        assert!(prompt.contains("不超过50字"));
    }

    #[test]
    fn test_parse_ai_response_valid_json() {
        let json = r#"{
            "tags": ["猫", "动物", "可爱", "宠物", "猫咪"],
            "description": "一只可爱的橘猫坐在窗台上晒太阳",
            "category": "动物",
            "confidence": 0.95
        }"#;

        let result = parse_ai_response(json, "test", "test-model").unwrap();
        assert_eq!(result.tags.len(), 5);
        assert!(result.tags.contains(&"猫".to_string()));
        assert_eq!(result.category, "动物");
        assert_eq!(result.confidence, 0.95);
        assert!(!result.description.is_empty());
    }

    #[test]
    fn test_parse_ai_response_missing_fields() {
        let json = r#"{
            "tags": ["test"],
            "description": "A test image"
        }"#;

        let result = parse_ai_response(json, "test", "test-model").unwrap();
        assert_eq!(result.tags, vec!["test"]);
        assert_eq!(result.description, "A test image");
        assert_eq!(result.category, "other");
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_parse_ai_response_invalid_json() {
        let result = parse_ai_response("not valid json", "test", "test-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_tc_ai_sp_002_timeout_configuration() {
        let config = LMStudioConfig::default();
        assert_eq!(config.timeout_secs, 60, "默认超时应为 60 秒");

        let client = LMStudioClient::new(config).unwrap();
        assert_eq!(client.config.timeout_secs, 60);

        let custom_config = LMStudioConfig {
            timeout_secs: 120,
            ..Default::default()
        };
        let client = LMStudioClient::new(custom_config).unwrap();
        assert_eq!(client.config.timeout_secs, 120);
    }

    #[test]
    fn test_tc_ai_sp_002_timeout_error_message() {
        let config = LMStudioConfig {
            timeout_secs: 1,
            ..Default::default()
        };
        let client = LMStudioClient::new(config).unwrap();
        assert_eq!(client.config.timeout_secs, 1);
    }

    #[test]
    fn test_tc_ai_sp_003_non_json_text_response() {
        let plain_text = "This is not a JSON response, just plain text from the AI model.";
        let result = parse_ai_response(plain_text, "test", "test-model");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("解析 AI JSON 响应失败"),
            "错误消息应包含解析失败描述，实际: {}",
            err_msg
        );
    }

    #[test]
    fn test_tc_ai_sp_003_html_error_response() {
        let html_response =
            r#"<html><body><h1>502 Bad Gateway</h1><p>Server Error</p></body></html>"#;
        let result = parse_ai_response(html_response, "test", "test-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_tc_ai_sp_003_partial_json_response() {
        let partial_json = r#"{"tags": ["猫", "动物"], "description": "一只猫"#;
        let result = parse_ai_response(partial_json, "test", "test-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_tc_ai_sp_003_empty_string_response() {
        let result = parse_ai_response("", "test", "test-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_tc_ai_sp_003_markdown_wrapped_json() {
        let markdown_json = r#"```json
{"tags": ["测试"], "description": "测试图片", "category": "其他", "confidence": 0.8}
```"#;
        let result = parse_ai_response(markdown_json, "test", "test-model");
        assert!(
            result.is_ok(),
            "Markdown 包裹的 JSON 应能被 strip_markdown_wrapper 清理后解析"
        );
        let ai_result = result.unwrap();
        assert_eq!(ai_result.tags, vec!["测试"]);
    }

    #[test]
    fn test_tc_ai_sp_003_json_with_wrong_types() {
        let wrong_types = r#"{"tags": "not_an_array", "description": 123, "category": true, "confidence": "high"}"#;
        let result = parse_ai_response(wrong_types, "test", "test-model");
        assert!(result.is_ok(), "字段类型错误时应使用默认值而非报错");
        let ai_result = result.unwrap();
        assert!(ai_result.tags.is_empty(), "tags 非数组时应为空");
        assert_eq!(ai_result.category, "other");
        assert_eq!(ai_result.confidence, 0.5);
    }
}
