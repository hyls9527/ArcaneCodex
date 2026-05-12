#![allow(missing_docs)]
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use xmpkit::core::namespace::ns;
use xmpkit::{XmpFile, XmpMeta, XmpOptions, XmpValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmpMetadata {
    pub creator: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub subject: Vec<String>,
    pub keywords: Vec<String>,
    pub rating: Option<i32>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
}

impl Default for XmpMetadata {
    fn default() -> Self {
        Self {
            creator: Some("ArcaneGallery".to_string()),
            title: None,
            description: None,
            subject: vec![],
            keywords: vec![],
            rating: None,
            created_date: None,
            modified_date: None,
        }
    }
}

pub struct XmpService;

impl XmpService {
    /// 从文件读取 XMP 元数据
    pub fn read_xmp_from_file(file_path: &Path) -> Result<Option<XmpMetadata>, String> {
        let mut file = XmpFile::new();
        file.open(file_path)
            .map_err(|e| format!("打开文件失败: {}", e))?;

        match file.get_xmp() {
            Some(meta) => {
                let result = Self::meta_to_struct(meta)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// 将 XMP 元数据写入文件
    pub fn write_xmp_to_file(file_path: &Path, metadata: &XmpMetadata) -> Result<(), String> {
        if !file_path.exists() {
            return Err(format!("文件不存在: {}", file_path.display()));
        }

        let data = fs::read(file_path).map_err(|e| format!("读取文件失败: {}", e))?;

        let mut file = XmpFile::new();
        file.from_bytes_with(&data, XmpOptions::default().for_update())
            .map_err(|e| format!("加载文件失败: {}", e))?;

        let meta = Self::struct_to_meta(metadata);
        file.put_xmp(meta);

        let output = file
            .write_to_bytes()
            .map_err(|e| format!("写入XMP失败: {}", e))?;

        fs::write(file_path, output).map_err(|e| format!("保存文件失败: {}", e))?;

        Ok(())
    }

    /// 创建 XMP Sidecar 文件
    pub fn create_xmp_sidecar(
        image_path: &Path,
        metadata: &XmpMetadata,
    ) -> Result<PathBuf, String> {
        let sidecar_path = image_path.with_extension("xmp");

        let meta = Self::struct_to_meta(metadata);
        let packet = meta
            .serialize()
            .map_err(|e| format!("序列化XMP失败: {}", e))?;

        fs::write(&sidecar_path, packet).map_err(|e| format!("写入Sidecar失败: {}", e))?;

        Ok(sidecar_path)
    }

    /// 从 XmpMeta 转换为结构体
    fn meta_to_struct(meta: &XmpMeta) -> Result<XmpMetadata, String> {
        let creator = meta
            .get_property(ns::XMP, "CreatorTool")
            .and_then(|v| match v {
                XmpValue::String(s) => Some(s.clone()),
                _ => None,
            });

        let title = meta.get_property(ns::DC, "title").and_then(|v| match v {
            XmpValue::String(s) => Some(s.clone()),
            _ => None,
        });

        let description = meta
            .get_property(ns::DC, "description")
            .and_then(|v| match v {
                XmpValue::String(s) => Some(s.clone()),
                _ => None,
            });

        let subject_raw = meta
            .get_property(ns::DC, "subject")
            .and_then(|v| match v {
                XmpValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let subject: Vec<String> = subject_raw
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let keywords_raw = meta
            .get_property("http://ns.adobe.com/pdf/1.3/", "Keywords")
            .and_then(|v| match v {
                XmpValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let keywords: Vec<String> = keywords_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(XmpMetadata {
            creator,
            title,
            description,
            subject,
            keywords,
            rating: None,
            created_date: None,
            modified_date: None,
        })
    }

    /// 从结构体转换为 XmpMeta
    fn struct_to_meta(metadata: &XmpMetadata) -> XmpMeta {
        let mut meta = XmpMeta::new();

        if let Some(ref c) = metadata.creator {
            let _ = meta.set_property(ns::DC, "creator", XmpValue::String(c.clone()));
        }
        if let Some(ref t) = metadata.title {
            let _ = meta.set_property(ns::DC, "title", XmpValue::String(t.clone()));
        }
        if let Some(ref d) = metadata.description {
            let _ = meta.set_property(ns::DC, "description", XmpValue::String(d.clone()));
        }
        if !metadata.subject.is_empty() {
            let _ = meta.set_property(
                ns::DC,
                "subject",
                XmpValue::String(metadata.subject.join("; ")),
            );
        }
        if !metadata.keywords.is_empty() {
            let _ = meta.set_property(
                "http://ns.adobe.com/pdf/1.3/",
                "Keywords",
                XmpValue::String(metadata.keywords.join(", ")),
            );
        }
        // 标记创建工具
        let _ = meta.set_property(
            ns::XMP,
            "CreatorTool",
            XmpValue::String("ArcaneGallery".to_string()),
        );

        meta
    }
}
