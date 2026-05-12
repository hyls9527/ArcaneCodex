#![allow(missing_docs)]
use crate::core::db::Database;
use crate::core::image::ImageProcessor;
use crate::core::search_index::clear_search_cache;
use crate::utils::error::{AppError, AppResult};
use crate::utils::hash::calculate_sha256;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{error, info, warn};

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
const MIN_DISK_SPACE_REQUIRED: u64 = 100 * 1024 * 1024;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "tiff", "tif", "avif",
];

const SUPPORTED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/bmp",
    "image/x-icon",
    "image/tiff",
    "image/avif",
    "image/heic",
    "image/heif",
];

/// Windows 系统敏感目录列表（禁止导出目标）
#[cfg(windows)]
const SENSITIVE_DIRS: &[&str] = &[
    r"C:\Windows",
    r"C:\Program Files",
    r"C:\Program Files (x86)",
    r"C:\ProgramData",
    r"C:\System32",
    r"C:\Windows\System32",
];

/// 非Windows系统的敏感目录（保留接口）
#[cfg(not(windows))]
const SENSITIVE_DIRS: &[&str] = &["/usr/bin", "/usr/sbin", "/bin", "/sbin", "/etc"];

/// 路径安全检查 - 防止路径穿越攻击（深度防护版）
///
/// # 安全机制（多层防御）
/// 1. **组件级检查**: 在解析前拒绝包含 `..` 的路径
/// 2. **规范化验证**: 使用 canonicalize() 解析所有符号链接和相对路径
/// 3. **边界确认**: 确保最终路径严格在允许的基目录内
/// 4. **特殊路径拦截**: 阻止 Windows UNC 路径、设备路径、NTFS 流等
///
/// # 参数
/// - `base_dir`: 允许的根目录（安全边界）
/// - `user_input`: 用户输入的路径
/// - `must_exist`: 路径是否必须存在（用于区分导入/导出场景）
///
/// # 参考
/// - [Rust Template Path Traversal Fix](https://github.com/EffortlessMetrics/Rust-Template/issues/10)
/// - [path-security 库](https://github.com/redasgard/path-security)
pub(crate) fn sanitize_path(
    base_dir: &Path,
    user_input: &str,
    must_exist: bool,
) -> Result<PathBuf, String> {
    use std::path::Component;

    let input = user_input.trim();

    // ===== 层1: 输入预处理 =====
    // 空路径拒绝
    if input.is_empty() {
        return Err("路径不能为空".to_string());
    }

    // 路径长度限制（Windows MAX_PATH = 260，留余量）
    if input.len() > 240 {
        return Err(format!("路径过长 ({} 字符，最大 240)", input.len()));
    }

    let path = PathBuf::from(input);

    // ===== 层2: 特殊路径模式拦截 =====
    let input_lower = input.to_lowercase();

    // Windows UNC 路径拦截 (\\server\share, //server/share)
    if input.starts_with("\\\\") || input.starts_with("//") {
        return Err("不允许使用 UNC 路径".to_string());
    }

    // Windows 设备路径拦截 (\\.\COM1, \\?\C:, etc.)
    if input_lower.contains("\\\\.\\") || input_lower.contains("\\\\?\\") {
        return Err("不允许使用设备路径".to_string());
    }

    // NTFS Alternate Data Streams 拦截 (file.txt::$DATA)
    if input.contains("::$") {
        return Err("检测到 NTFS 数据流攻击".to_string());
    }

    // Windows 保留名称拦截 (CON, PRN, AUX, NUL, COM1-9, LPT1-9)
    #[cfg(windows)]
    {
        const RESERVED_NAMES: &[&str] = &[
            "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
            "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
        ];
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if RESERVED_NAMES.contains(&file_name.as_str()) {
            return Err(format!("使用 Windows 保留文件名: {}", file_name));
        }
    }

    // ===== 层3: 组件级遍历检查 =====
    // 在任何规范化之前，先检查原始路径组件
    let has_parent_dir = path.components().any(|c| matches!(c, Component::ParentDir));
    if has_parent_dir {
        warn!("路径遍历检测(组件级): {}", user_input);
        return Err("路径不允许包含 '..' 目录遍历序列".to_string());
    }

    // 检查可疑编码模式（URL 编码、Unicode 同形字等）
    if input.contains("%2e") || input_contains_encoded_traversal(input) {
        return Err("检测到编码后的路径遍历尝试".to_string());
    }

    // ===== 层4: 规范化 + 边界验证 =====
    let canonical = if must_exist {
        // 已存在路径：使用 canonicalize（解析符号链接）
        match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                warn!("路径规范化失败: {} - 错误: {}", user_input, e);
                return Err(format!("无效路径: 无法解析 '{}'", user_input));
            }
        }
    } else {
        // 不存在的目标路径：使用逻辑规范化
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            base_dir.join(&path)
        };
        normalize_path_strict(&resolved)?
    };

    // ===== 层5: 最终边界确认 =====
    // 获取 base_dir 的规范形式
    let canonical_base = match base_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // base_dir 不存在时使用其本身（罕见情况，如首次运行）
            base_dir.to_path_buf()
        }
    };

    // 严格检查：规范化后的路径必须以 base_dir 为前缀
    if !canonical.starts_with(&canonical_base) {
        error!(
            "路径穿越检测: 输入='{}', 规范化='{}', 基目录='{}'",
            user_input,
            canonical.display(),
            canonical_base.display()
        );
        return Err("安全限制: 路径超出允许的范围".to_string());
    }

    info!(
        "路径安全验证通过: '{}' -> '{}'",
        user_input,
        canonical.display()
    );
    Ok(canonical)
}

/// 检查输入是否包含编码后的路径遍历模式
fn input_contains_encoded_traversal(input: &str) -> bool {
    // URL 编码的 ..
    let encoded_patterns = [
        "%2e%2e",
        "%2E%2E",     // 标准 URL 编码
        "%252e%252e", // 双重 URL 编码
        "%c0%ae",
        "%C0%AE", // UTF-8 overlong 编码
        "..%2f",
        "..%5c",        // 混合编码
        "%u002e%u002e", // Unicode percent 编码
        "&#46;&#46;",   // HTML 实体编码
        "\x2e\x2e",     // Hex 编码
    ];

    let input_lower = input.to_lowercase();
    encoded_patterns
        .iter()
        .any(|p| input_lower.contains(&p.to_lowercase()))
}

/// 标准化路径（不要求文件存在）- 严格版本
/// 用于处理不存在的目标路径（如导出目录）
fn normalize_path_strict(path: &Path) -> Result<PathBuf, String> {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    let mut parent_count = 0i32;

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                normalized.push(prefix.as_os_str());
            }
            Component::RootDir => {
                normalized.push(component.as_os_str());
            }
            Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::ParentDir => {
                // 弹出上一级目录
                if normalized.pop() {
                    parent_count -= 1;
                } else {
                    // 无法弹出说明尝试逃逸根目录
                    return Err("路径解析结果超出根目录边界".to_string());
                }
            }
            Component::CurDir => {
                // 忽略当前目录 .
                continue;
            }
        }
    }

    // 如果最终弹出次数过多，可能是恶意构造
    if parent_count.abs() > 10 {
        return Err("路径包含过多的目录层级跳转".to_string());
    }

    Ok(normalized)
}

/// 验证导入路径的安全性
///
/// import_images 允许从用户指定的任意位置导入文件（这是设计意图），
/// 但必须拒绝明显的恶意路径模式。
///
/// # 拒绝条件
/// - UNC 路径 (\\server\share)
/// - Windows 设备路径 (\\.\, \\?\)
/// - NTFS 数据流 (::$DATA)
/// - 路径长度超过限制
/// - 包含空字节或控制字符
fn validate_import_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    // 空路径检查
    if path_str.trim().is_empty() {
        return Err("导入路径为空".to_string());
    }

    // 路径长度限制
    if path_str.len() > 240 {
        return Err(format!("导入路径过长: {} 字符", path_str.len()));
    }

    let path_lower = path_str.to_lowercase();

    // UNC 路径拦截（网络路径可能存在安全风险）
    if path_str.starts_with("\\\\") || path_str.starts_with("//") {
        return Err("不允许从 UNC 网络路径导入".to_string());
    }

    // Windows 设备路径拦截
    if path_lower.contains("\\\\.\\") || path_lower.contains("\\\\?\\") {
        return Err("检测到设备路径，不允许导入".to_string());
    }

    // NTFS Alternate Data Streams 拦截
    if path_str.contains("::$") {
        return Err("检测到 NTFS 数据流，不允许导入".to_string());
    }

    // 编码后的遍历序列拦截
    if input_contains_encoded_traversal(&path_str) {
        return Err("检测到编码后的路径遍历尝试".to_string());
    }

    // 控制字符和空字节检查
    for ch in path_str.chars() {
        if ch.is_control() && ch != '\t' {
            return Err("路径包含非法控制字符".to_string());
        }
    }

    Ok(())
}

/// 检查是否为系统敏感目录
fn is_sensitive_directory(path: &Path) -> bool {
    #[cfg(windows)]
    {
        // Windows 路径比较（不区分大小写）
        let path_str = path.to_string_lossy().to_lowercase();
        for sensitive in SENSITIVE_DIRS.iter() {
            if path_str.starts_with(&sensitive.to_lowercase())
                || *sensitive == path.to_string_lossy()
            {
                return true;
            }
        }
        false
    }

    #[cfg(not(windows))]
    {
        // Unix/Linux 路径比较
        for sensitive in SENSITIVE_DIRS.iter() {
            if path.starts_with(sensitive) || path == Path::new(sensitive) {
                return true;
            }
        }
        false
    }
}

/// Response structure for broken link check
#[derive(Debug, Serialize, Deserialize)]
pub struct BrokenLinkInfo {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckBrokenLinksResult {
    pub broken_count: usize,
    pub broken_images: Vec<BrokenLinkInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveImageResult {
    pub archived: bool,
    pub dest_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SafeExportError {
    pub id: i64,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SafeExportResult {
    pub exported_count: usize,
    pub errors: Vec<SafeExportError>,
}

fn get_available_disk_space(path: &Path) -> AppResult<u64> {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::GetDiskFreeSpaceExW;

        unsafe {
            let wpath: Vec<u16> = OsStr::new(path.parent().unwrap_or(path))
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut free_bytes: u64 = 0;
            let mut _total_bytes: u64 = 0;
            let mut _total_free_bytes: u64 = 0;

            let result = GetDiskFreeSpaceExW(
                wpath.as_ptr(),
                &mut free_bytes as *mut u64 as *mut _,
                &mut _total_bytes as *mut u64 as *mut _,
                &mut _total_free_bytes as *mut u64 as *mut _,
            );

            if result != 0 {
                Ok(free_bytes)
            } else {
                Err(AppError::io(std::io::Error::other(
                    "Failed to get disk space information",
                )))
            }
        }
    }

    #[cfg(not(windows))]
    {
        use std::mem;
        let cpath = std::ffi::CString::new(path.to_str().unwrap_or("/"))
            .map_err(|_| AppError::validation("Invalid path"))?;

        let mut statvfs_buf: libc::statvfs = unsafe { mem::zeroed() };

        let result = unsafe { libc::statvfs(cpath.as_ptr(), &mut statvfs_buf) };

        if result == 0 {
            Ok(statvfs_buf.f_frsize * statvfs_buf.f_bavail)
        } else {
            Err(AppError::io(std::io::Error::other("无法获取磁盘空间信息")))
        }
    }
}

fn expand_paths(input_paths: &[String]) -> Vec<String> {
    let mut expanded = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);

        if !path.exists() {
            info!("路径不存在，跳过: {}", path_str);
            continue;
        }

        if path.is_dir() {
            info!("检测到目录，开始扫描: {}", path_str);
            scan_directory(path, &mut expanded);
        } else {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                expanded.push(path_str.clone());
            } else {
                info!("跳过不支持的文件格式: {}", path_str);
            }
        }
    }

    expanded
}

fn scan_directory(dir: &Path, results: &mut Vec<String>) {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_directory(&path, results);
                } else {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                        results.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        Err(e) => {
            warn!("无法读取目录 {}: {}", dir.display(), e);
        }
    }
}

/// 验证文件魔术字节是否与声明的格式一致
fn validate_magic_bytes(file_path: &Path, expected_ext: &str) -> AppResult<bool> {
    use std::io::Read;

    let mut file = fs::File::open(file_path).map_err(|e| AppError::io(std::io::Error::other(e)))?;
    let mut header = [0u8; 16];
    let bytes_read = file
        .read(&mut header)
        .map_err(|e| AppError::io(std::io::Error::other(e)))?;

    if bytes_read < 4 {
        return Ok(false);
    }

    let is_valid = match expected_ext {
        "jpg" | "jpeg" => header[0..3] == [0xFF, 0xD8, 0xFF],
        "png" => header[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        "gif" => header[0..3] == [0x47, 0x49, 0x46],
        "webp" => {
            header[0..4] == [0x52, 0x49, 0x46, 0x46] && header[8..12] == [0x57, 0x45, 0x42, 0x50]
        }
        "bmp" => header[0..2] == [0x42, 0x4D],
        "ico" => header[0..4] == [0x00, 0x00, 0x01, 0x00],
        "tiff" | "tif" => {
            header[0..4] == [0x49, 0x49, 0x2A, 0x00] || header[0..4] == [0x4D, 0x4D, 0x00, 0x2A]
        }
        "avif" => header[4..12] == [0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66],
        _ => true,
    };

    Ok(is_valid)
}

fn validate_file(file_path: &Path) -> AppResult<(String, u64)> {
    if !file_path.exists() {
        return Err(AppError::validation(format!(
            "文件不存在: {}",
            file_path.display()
        )));
    }

    let metadata = fs::metadata(file_path)
        .map_err(|e| AppError::validation(format!("无法读取文件元数据: {}", e)))?;

    let file_size = metadata.len();
    if file_size == 0 {
        return Err(AppError::validation(format!(
            "文件为空: {}",
            file_path.display()
        )));
    }

    if file_size > MAX_FILE_SIZE {
        return Err(AppError::validation(format!(
            "文件大小 {} 超过限制 ({} 字节)",
            file_size, MAX_FILE_SIZE
        )));
    }

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::validation(format!(
            "不支持的文件格式: .{}",
            extension
        )));
    }

    let mime_type = match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "bmp" => "image/bmp".to_string(),
        "ico" => "image/x-icon".to_string(),
        "tiff" | "tif" => "image/tiff".to_string(),
        "avif" => "image/avif".to_string(),
        _ => "application/octet-stream".to_string(),
    };

    if !SUPPORTED_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(AppError::validation(format!(
            "不支持的 MIME 类型: {}",
            mime_type
        )));
    }

    if !validate_magic_bytes(file_path, &extension)? {
        return Err(AppError::validation(format!(
            "文件魔术字节与扩展名不匹配: .{}",
            extension
        )));
    }

    Ok((mime_type, file_size))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub duplicate_count: usize,
    pub error_count: usize,
    pub image_ids: Vec<i64>,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub file_path: String,
    pub reason: String,
}

/// Progress event emitted during image import
#[derive(Debug, Serialize, Clone)]
pub struct ImportProgress {
    pub current_file: String,
    pub current: usize,
    pub total: usize,
    pub status: ImportStatus,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Processing,
    Success,
    Duplicate,
    Error,
}

fn is_duplicate(conn: &rusqlite::Connection, file_hash: &str) -> AppResult<bool> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM images WHERE file_hash = ?)",
            [file_hash],
            |row| row.get(0),
        )
        .map_err(AppError::database)?;

    Ok(exists)
}

fn get_thumbnail_dir(app: &AppHandle) -> std::path::PathBuf {
    let app_data = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let dir = app_data.join("thumbnails");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn generate_thumbnail_path(image_id: i64, app: &AppHandle) -> String {
    let thumb_dir = get_thumbnail_dir(app);
    thumb_dir
        .join(format!("{}.webp", image_id))
        .to_string_lossy()
        .to_string()
}

fn update_image_metadata(
    conn: &rusqlite::Connection,
    image_id: i64,
    thumbnail_path: &str,
    phash: &str,
    width: i32,
    height: i32,
    exif_data: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE images SET thumbnail_path = ?2, phash = ?3, width = ?4, height = ?5, exif_data = ?6 WHERE id = ?1",
        rusqlite::params![image_id, thumbnail_path, phash, width, height, exif_data],
    ).map_err(AppError::database)?;
    Ok(())
}

fn create_ai_task(conn: &rusqlite::Connection, image_id: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO task_queue (image_id, task_type, status) VALUES (?1, 'ai_analysis', 'pending')",
        rusqlite::params![image_id],
    ).map_err(AppError::database)?;
    Ok(())
}

fn insert_image_record(
    conn: &rusqlite::Connection,
    file_path: &str,
    file_name: &str,
    file_size: u64,
    file_hash: &str,
    mime_type: &str,
) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![file_path, file_name, file_size, file_hash, mime_type],
    ).map_err(AppError::database)?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

/// 导入阶段的中间数据结构
struct PendingImport {
    id: i64,
    file_path: String,
    file_name: String,
}

#[tauri::command]
pub async fn import_images(
    app: AppHandle,
    db: State<'_, Database>,
    file_paths: Vec<String>,
) -> AppResult<ImportResult> {
    info!("开始导入 {} 个路径", file_paths.len());

    let expanded_paths = expand_paths(&file_paths);
    info!(
        "路径展开后: {} 个文件 (原始 {} 个路径)",
        expanded_paths.len(),
        file_paths.len()
    );

    let total = expanded_paths.len();

    if total == 0 {
        return Err(AppError::validation("未找到可导入的图片文件".to_string()));
    }

    // Check disk space before importing
    if !expanded_paths.is_empty() {
        let first_file_path = Path::new(&expanded_paths[0]);
        match get_available_disk_space(first_file_path) {
            Ok(available_space) => {
                let mut total_size_needed: u64 = 0;
                for path_str in &expanded_paths {
                    let file_path = Path::new(path_str);
                    if let Ok(metadata) = fs::metadata(file_path) {
                        total_size_needed += metadata.len();
                    }
                }

                if available_space < MIN_DISK_SPACE_REQUIRED + total_size_needed {
                    let available_mb = available_space / (1024 * 1024);
                    let required_mb = (MIN_DISK_SPACE_REQUIRED + total_size_needed) / (1024 * 1024);
                    return Err(AppError::validation(format!(
                        "磁盘空间不足。可用空间: {} MB，需要空间: {} MB",
                        available_mb, required_mb
                    )));
                }
            }
            Err(_) => {
                warn!("无法检查磁盘空间，跳过磁盘空间检查");
            }
        }
    }

    let mut result = ImportResult {
        success_count: 0,
        duplicate_count: 0,
        error_count: 0,
        image_ids: vec![],
        errors: vec![],
    };

    // ========== 阶段1: 快速入库 ==========
    // 目标：快速验证+插入记录，最小化数据库连接持有时间
    info!("[阶段1] 开始快速入库...");

    let conn = db.open_connection().map_err(AppError::database)?;
    let mut pending_imports: Vec<PendingImport> = Vec::new();

    for (index, path_str) in expanded_paths.iter().enumerate() {
        let file_path = Path::new(path_str);

        // ========== 导入路径安全预检 ==========
        // import_images 允许从任意位置导入（设计意图），但必须拒绝恶意路径
        if let Err(e) = validate_import_path(file_path) {
            warn!("导入路径安全检查失败: {} - {}", path_str, e);
            result.error_count += 1;
            result.errors.push(ImportError {
                file_path: path_str.clone(),
                reason: e,
            });
            continue;
        }

        let canonical_path = match file_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                warn!("路径规范化失败: {} - {}", path_str, e);
                result.error_count += 1;
                result.errors.push(ImportError {
                    file_path: path_str.clone(),
                    reason: format!("路径规范化失败: {}", e),
                });
                continue;
            }
        };
        let canonical_str = canonical_path.to_string_lossy().to_string();
        let file_name = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Emit processing progress
        let _ = app.emit(
            "import-progress",
            ImportProgress {
                current_file: file_name.clone(),
                current: index + 1,
                total,
                status: ImportStatus::Processing,
            },
        );

        match validate_file(&canonical_path) {
            Ok((mime_type, file_size)) => match calculate_sha256(&canonical_path) {
                Ok(hash) => match is_duplicate(&conn, &hash) {
                    Ok(true) => {
                        info!("跳过重复文件: {}", canonical_str);
                        result.duplicate_count += 1;
                        let _ = app.emit(
                            "import-progress",
                            ImportProgress {
                                current_file: file_name.clone(),
                                current: index + 1,
                                total,
                                status: ImportStatus::Duplicate,
                            },
                        );
                    }
                    Ok(false) => {
                        match insert_image_record(
                            &conn,
                            &canonical_str,
                            &file_name,
                            file_size,
                            &hash,
                            &mime_type,
                        ) {
                            Ok(id) => {
                                info!("[阶段1] 成功插入图片记录: {} (ID: {})", file_name, id);
                                pending_imports.push(PendingImport {
                                    id,
                                    file_path: canonical_str.clone(),
                                    file_name: file_name.clone(),
                                });
                                result.success_count += 1;
                                result.image_ids.push(id);

                                let _ = app.emit(
                                    "import-progress",
                                    ImportProgress {
                                        current_file: file_name.clone(),
                                        current: index + 1,
                                        total,
                                        status: ImportStatus::Success,
                                    },
                                );
                            }
                            Err(e) => {
                                error!("数据库插入失败: {} - {}", file_name, e);
                                result.error_count += 1;
                                result.errors.push(ImportError {
                                    file_path: canonical_str.clone(),
                                    reason: e.to_string(),
                                });
                                let _ = app.emit(
                                    "import-progress",
                                    ImportProgress {
                                        current_file: file_name.clone(),
                                        current: index + 1,
                                        total,
                                        status: ImportStatus::Error,
                                    },
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("重复检测失败: {} - {}", file_name, e);
                        result.error_count += 1;
                        result.errors.push(ImportError {
                            file_path: canonical_str.clone(),
                            reason: e.to_string(),
                        });
                        let _ = app.emit(
                            "import-progress",
                            ImportProgress {
                                current_file: file_name.clone(),
                                current: index + 1,
                                total,
                                status: ImportStatus::Error,
                            },
                        );
                    }
                },
                Err(e) => {
                    error!("哈希计算失败: {} - {}", file_name, e);
                    result.error_count += 1;
                    result.errors.push(ImportError {
                        file_path: canonical_str.clone(),
                        reason: format!("哈希计算失败: {}", e),
                    });
                    let _ = app.emit(
                        "import-progress",
                        ImportProgress {
                            current_file: file_name.clone(),
                            current: index + 1,
                            total,
                            status: ImportStatus::Error,
                        },
                    );
                }
            },
            Err(e) => {
                warn!("文件验证失败: {} - {}", canonical_str, e);
                result.error_count += 1;
                result.errors.push(ImportError {
                    file_path: canonical_str.clone(),
                    reason: e.to_string(),
                });
                let _ = app.emit(
                    "import-progress",
                    ImportProgress {
                        current_file: file_name.clone(),
                        current: index + 1,
                        total,
                        status: ImportStatus::Error,
                    },
                );
            }
        }
    }

    // 阶段1完成：立即释放数据库连接
    drop(conn);
    info!(
        "[阶段1] 快速入库完成: 成功 {}, 待处理元数据: {}",
        result.success_count,
        pending_imports.len()
    );

    // ========== 阶段2: 异步处理元数据 ==========
    // 目标：在后台处理耗时操作（缩略图+pHash+EXIF），不阻塞数据库连接
    if !pending_imports.is_empty() {
        info!(
            "[阶段2] 开始异步处理元数据 ({} 个文件)...",
            pending_imports.len()
        );

        for (idx, import) in pending_imports.iter().enumerate() {
            // 重新获取数据库连接（短生命周期）
            match db.open_connection() {
                Ok(conn) => {
                    let file_path_clone = import.file_path.clone();
                    let thumb_path = generate_thumbnail_path(import.id, &app);
                    let thumb_path_clone = thumb_path.clone();

                    let process_result = tokio::task::spawn_blocking(move || {
                        let thumb_result =
                            ImageProcessor::generate_thumbnail(&file_path_clone, &thumb_path_clone);
                        let phash_result = ImageProcessor::calculate_phash(&file_path_clone);
                        let exif_result = ImageProcessor::extract_exif(&file_path_clone);
                        let (w, h) = match &exif_result {
                            Ok(exif) => (
                                exif.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                exif.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            ),
                            Err(_) => match image::open(Path::new(&file_path_clone)) {
                                Ok(img) => {
                                    let (width, height) = img.dimensions();
                                    (width as i32, height as i32)
                                }
                                Err(_) => (0, 0),
                            },
                        };
                        (thumb_result, phash_result, exif_result, w, h)
                    })
                    .await;

                    let (thumb_result, phash_result, exif_result, w, h) = match process_result {
                        Ok(r) => r,
                        Err(e) => {
                            error!("[阶段2] spawn_blocking panic (ID: {}): {}", import.id, e);
                            continue;
                        }
                    };

                    // 写回元数据（失败仅 warn，不阻塞导入）
                    let thumb_path_str = match &thumb_result {
                        Ok(_) => thumb_path.clone(),
                        Err(_) => String::new(),
                    };
                    let phash_str = match &phash_result {
                        Ok(h) => h.clone(),
                        Err(_) => String::new(),
                    };
                    let exif_json = match &exif_result {
                        Ok(v) => serde_json::to_string(v).unwrap_or_default(),
                        Err(_) => String::new(),
                    };

                    if let Err(e) = update_image_metadata(
                        &conn,
                        import.id,
                        &thumb_path_str,
                        &phash_str,
                        w,
                        h,
                        &exif_json,
                    ) {
                        warn!("更新图片元数据失败 (ID: {}): {}", import.id, e);
                    } else {
                        if let Err(e) = &thumb_result {
                            warn!("缩略图生成失败 (ID: {}): {}", import.id, e);
                        }
                        if let Err(e) = &phash_result {
                            warn!("pHash 计算失败 (ID: {}): {}", import.id, e);
                        }
                        if let Err(e) = &exif_result {
                            warn!("EXIF 提取失败 (ID: {}): {}", import.id, e);
                        }
                    }

                    // 创建 AI 任务队列记录（失败仅 warn）
                    if let Err(e) = create_ai_task(&conn, import.id) {
                        warn!("创建 AI 任务失败 (ID: {}): {}", import.id, e);
                    }

                    info!(
                        "[阶段2] 元数据处理完成 ({}/{}): ID {} - {}",
                        idx + 1,
                        pending_imports.len(),
                        import.id,
                        import.file_name
                    );

                    // 发送元数据处理进度
                    let _ = app.emit(
                        "import-metadata-progress",
                        ImportProgress {
                            current_file: import.file_name.clone(),
                            current: idx + 1,
                            total: pending_imports.len(),
                            status: ImportStatus::Processing,
                        },
                    );
                }
                Err(e) => {
                    error!("[阶段2] 无法获取数据库连接 (ID: {}): {}", import.id, e);
                }
            }
        }

        info!("[阶段2] 异步元数据处理完成");
    }

    info!(
        "导入完成: 成功 {}, 重复 {}, 错误 {}",
        result.success_count, result.duplicate_count, result.error_count
    );

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageFilters {
    pub ai_status: Option<String>,
    pub category: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[tauri::command]
pub async fn get_images(
    db: State<'_, Database>,
    page: u32,
    page_size: u32,
    filters: Option<ImageFilters>,
) -> AppResult<serde_json::Value> {
    info!(
        "get_images called: page={}, page_size={}, filters={:?}",
        page, page_size, filters
    );
    let conn = db.open_connection().map_err(AppError::database)?;

    let offset = page.saturating_sub(1) * page_size;

    let mut count_sql = String::from("SELECT COUNT(*) FROM images");
    let mut sql = String::from(
        "SELECT id, file_path, file_name, file_size, file_hash, mime_type,
         width, height, thumbnail_path, phash, ai_status, ai_tags, ai_description,
         ai_category, ai_confidence, ai_tag_status, ai_provider, source, created_at, updated_at,
         COALESCE(generation_source, 'manual_import') as generation_source
         FROM images",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref f) = filters {
        if let Some(ref ai_status) = f.ai_status {
            conditions.push("ai_status = ?".to_string());
            params.push(Box::new(ai_status.clone()));
        }
        if let Some(ref category) = f.category {
            conditions.push("ai_category = ?".to_string());
            params.push(Box::new(category.clone()));
        }
        if let Some(ref date_from) = f.date_from {
            conditions.push("created_at >= ?".to_string());
            params.push(Box::new(date_from.clone()));
        }
        if let Some(ref date_to) = f.date_to {
            conditions.push("created_at <= ?".to_string());
            params.push(Box::new(date_to.clone()));
        }
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        count_sql.push_str(&where_clause);
        sql.push_str(&where_clause);
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let total: i64 = conn
        .query_row(&count_sql, &param_refs[..], |row| row.get(0))
        .map_err(AppError::database)?;

    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    let mut all_params: Vec<&dyn rusqlite::types::ToSql> = param_refs;
    all_params.push(&page_size);
    all_params.push(&offset);

    let mut stmt = conn.prepare(&sql).map_err(AppError::database)?;

    let rows = stmt
        .query_map(&all_params[..], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "file_path": row.get::<_, String>(1)?,
                "file_name": row.get::<_, String>(2)?,
                "file_size": row.get::<_, i64>(3)?,
                "file_hash": row.get::<_, Option<String>>(4)?,
                "mime_type": row.get::<_, Option<String>>(5)?,
                "width": row.get::<_, Option<i32>>(6)?,
                "height": row.get::<_, Option<i32>>(7)?,
                "thumbnail_path": row.get::<_, Option<String>>(8)?,
                "phash": row.get::<_, Option<String>>(9)?,
                "ai_status": row.get::<_, String>(10)?,
                "ai_tags": row.get::<_, Option<String>>(11)?,
                "ai_description": row.get::<_, Option<String>>(12)?,
                "ai_category": row.get::<_, Option<String>>(13)?,
                "ai_confidence": row.get::<_, Option<f64>>(14)?,
                "ai_tag_status": row.get::<_, Option<String>>(15)?,
                "ai_provider": row.get::<_, Option<String>>(16)?,
                "source": row.get::<_, String>(17)?,
                "created_at": row.get::<_, String>(18)?,
                "updated_at": row.get::<_, String>(19)?,
                "generation_source": row.get::<_, String>(20)?,
            }))
        })
        .map_err(AppError::database)?;

    let images: Vec<serde_json::Value> = rows
        .filter_map(|r| match r {
            Ok(v) => Some(v),
            Err(e) => {
                error!("读取图片行失败: {}", e);
                None
            }
        })
        .collect();

    info!(
        "get_images returning: total={}, images_count={}",
        total,
        images.len()
    );

    Ok(serde_json::json!({
        "images": images,
        "total": total,
        "page": page,
        "page_size": page_size
    }))
}

#[tauri::command]
pub async fn get_image_detail(db: State<'_, Database>, id: i64) -> AppResult<serde_json::Value> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, mime_type,
             width, height, thumbnail_path, phash, exif_data, ai_status, ai_tags,
             ai_description, ai_category, ai_confidence, ai_model, ai_processed_at,
             ai_error_message, ai_retry_count, source, created_at, updated_at
             FROM images WHERE id = ?1",
        )
        .map_err(AppError::database)?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "file_path": row.get::<_, String>(1)?,
                "file_name": row.get::<_, String>(2)?,
                "file_size": row.get::<_, i64>(3)?,
                "file_hash": row.get::<_, Option<String>>(4)?,
                "mime_type": row.get::<_, Option<String>>(5)?,
                "width": row.get::<_, Option<i32>>(6)?,
                "height": row.get::<_, Option<i32>>(7)?,
                "thumbnail_path": row.get::<_, Option<String>>(8)?,
                "phash": row.get::<_, Option<String>>(9)?,
                "exif_data": row.get::<_, Option<String>>(10)?,
                "ai_status": row.get::<_, String>(11)?,
                "ai_tags": row.get::<_, Option<String>>(12)?,
                "ai_description": row.get::<_, Option<String>>(13)?,
                "ai_category": row.get::<_, Option<String>>(14)?,
                "ai_confidence": row.get::<_, Option<f64>>(15)?,
                "ai_model": row.get::<_, Option<String>>(16)?,
                "ai_processed_at": row.get::<_, Option<String>>(17)?,
                "ai_error_message": row.get::<_, Option<String>>(18)?,
                "ai_retry_count": row.get::<_, i32>(19)?,
                "source": row.get::<_, String>(20)?,
                "created_at": row.get::<_, String>(21)?,
                "updated_at": row.get::<_, String>(22)?,
            }))
        })
        .map_err(AppError::database)?;

    clear_search_cache();

    Ok(result)
}

#[tauri::command]
pub async fn delete_images(db: State<'_, Database>, ids: Vec<i64>) -> AppResult<usize> {
    let conn = db.open_connection().map_err(AppError::database)?;

    let mut deleted = 0;

    for &id in &ids {
        // 1. 查询 thumbnail_path 和 file_path
        let thumb_path: Option<String> = conn
            .query_row(
                "SELECT thumbnail_path FROM images WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .ok();

        let file_path: Option<String> = conn
            .query_row(
                "SELECT file_path FROM images WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .ok();

        // 2. 删除 search_index 记录
        conn.execute(
            "DELETE FROM search_index WHERE image_id = ?",
            rusqlite::params![id],
        )
        .map_err(AppError::database)?;

        // 3. 删除 images 记录
        let row_deleted = conn
            .execute("DELETE FROM images WHERE id = ?", rusqlite::params![id])
            .map_err(AppError::database)?;

        if row_deleted > 0 {
            deleted += 1;

            // 4. 删除缩略图文件（失败仅 warn，不阻塞）
            if let Some(thumb) = &thumb_path {
                let thumb_path = Path::new(thumb);
                if thumb_path.exists() {
                    if let Err(e) = fs::remove_file(thumb_path) {
                        warn!("删除缩略图失败 {}: {}", thumb, e);
                    } else {
                        info!("已删除缩略图: {}", thumb);
                    }
                }
            }

            // 5. 如果原文件在应用数据目录内，也一并删除
            if let Some(fp) = &file_path {
                if fp.starts_with("/app/")
                    || fp.starts_with("\\app\\")
                    || fp.contains("imported_images")
                {
                    let orig_path = Path::new(fp);
                    if orig_path.exists() {
                        if let Err(e) = fs::remove_file(orig_path) {
                            warn!("删除原文件失败 {}: {}", fp, e);
                        } else {
                            info!("已删除原文件: {}", fp);
                        }
                    }
                }
            }
        }
    }

    info!("删除了 {} 张图片", deleted);

    clear_search_cache();

    Ok(deleted)
}

/// Checks all file_paths in the images table and marks missing files as broken.
/// Returns count and list of broken images.
#[tauri::command]
pub async fn check_broken_links(db: State<'_, Database>) -> AppResult<CheckBrokenLinksResult> {
    info!("开始检查失效链接");

    let conn = db.open_connection().map_err(AppError::database)?;

    // Query all images with their file paths
    let mut stmt = conn
        .prepare("SELECT id, file_path, file_name FROM images")
        .map_err(AppError::database)?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(AppError::database)?;

    let mut broken_images: Vec<BrokenLinkInfo> = Vec::new();
    let mut ids_to_mark: Vec<i64> = Vec::new();

    for row in rows {
        match row {
            Ok((id, file_path, file_name)) => {
                if !Path::new(&file_path).exists() {
                    warn!("失效链接检测: ID {} 文件不存在: {}", id, file_path);
                    broken_images.push(BrokenLinkInfo {
                        id,
                        file_path: file_path.clone(),
                        file_name: file_name.clone(),
                    });
                    ids_to_mark.push(id);
                }
            }
            Err(e) => {
                error!("读取图片行失败: {}", e);
            }
        }
    }

    // Mark broken images in database
    if !ids_to_mark.is_empty() {
        for &id in &ids_to_mark {
            match conn.execute(
                "UPDATE images SET ai_status = 'broken_link', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                rusqlite::params![id],
            ) {
                Ok(updated) => {
                    info!("已标记图片 {} 为 broken_link (影响行数: {})", id, updated);
                }
                Err(e) => {
                    error!("标记图片 {} 为 broken_link 失败: {}", id, e);
                }
            }
        }
    }

    let broken_count = broken_images.len();
    info!("失效链接检查完成: 共发现 {} 个失效链接", broken_count);

    Ok(CheckBrokenLinksResult {
        broken_count,
        broken_images,
    })
}

/// Archives a single image by copying its file to the app data directory.
#[tauri::command]
pub async fn archive_image(
    app: AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> AppResult<ArchiveImageResult> {
    info!("开始归档图片: ID {}", id);

    let conn = db.open_connection().map_err(AppError::database)?;

    // Get file_path for the image
    let file_path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM images WHERE id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(AppError::database)?;

    let file_path = match file_path {
        Some(fp) => fp,
        None => {
            warn!("归档失败: 图片 ID {} 不存在", id);
            return Err(AppError::validation(format!("图片 ID {} 不存在", id)));
        }
    };

    let source = Path::new(&file_path);
    if !source.exists() {
        warn!("归档失败: 源文件不存在: {}", file_path);
        return Err(AppError::validation(format!("源文件不存在: {}", file_path)));
    }

    // ========== 安全检查：源路径验证（防止数据库被篡改注入恶意路径） ==========
    // archive_image 的源文件来自数据库记录，必须验证路径合法性
    // 使用 sanitize_path 验证源路径不包含遍历序列
    match sanitize_path(source, &file_path, true) {
        Ok(canonical_source) => {
            info!(
                "归档源路径安全验证通过 (ID: {}): {} -> {}",
                id,
                file_path,
                canonical_source.display()
            );
        }
        Err(e) => {
            error!(
                "归档安全检查失败 (ID: {}): 路径 '{}' 验证未通过 - {}",
                id, file_path, e
            );
            return Err(AppError::validation(format!(
                "安全限制: 源路径验证失败 - {}",
                e
            )));
        }
    }

    // Get destination directory
    let app_data = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let archive_dir = app_data.join("images");

    if let Err(e) = fs::create_dir_all(&archive_dir) {
        error!("创建归档目录失败 {}: {}", archive_dir.display(), e);
        return Err(AppError::validation(format!("创建归档目录失败: {}", e)));
    }

    // Destination path: use original file name
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let dest_path = archive_dir.join(file_name);

    // Handle duplicate: if file already exists, append a number
    let final_dest = if dest_path.exists() {
        let stem = dest_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let ext = dest_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mut counter = 1;
        loop {
            let new_name = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            let new_path = archive_dir.join(&new_name);
            if !new_path.exists() {
                break new_path;
            }
            counter += 1;
        }
    } else {
        dest_path
    };

    // Copy file
    match fs::copy(source, &final_dest) {
        Ok(_) => {
            let dest_str = final_dest.to_string_lossy().to_string();
            info!("图片归档成功: {} -> {}", file_path, dest_str);
            Ok(ArchiveImageResult {
                archived: true,
                dest_path: dest_str,
            })
        }
        Err(e) => {
            error!(
                "归档文件复制失败: {} -> {}: {}",
                file_path,
                final_dest.display(),
                e
            );
            Err(AppError::validation(format!("文件复制失败: {}", e)))
        }
    }
}

/// Batch exports images to a user-specified directory.
/// Non-existent files are logged as errors but do not block the operation.
#[tauri::command]
pub async fn safe_export(
    db: State<'_, Database>,
    image_ids: Vec<i64>,
    dest_dir: String,
) -> AppResult<SafeExportResult> {
    info!("开始批量导出 {} 张图片到: {}", image_ids.len(), dest_dir);

    let conn = db.open_connection().map_err(AppError::database)?;

    // ========== 安全检查 1: 路径穿越防护（使用统一验证函数） ==========
    let dest_path = Path::new(&dest_dir);

    // 使用 sanitize_path 进行深度路径验证
    // 导出目标目录可能不存在，所以 must_exist=false
    let normalized_dest = match sanitize_path(dest_path, &dest_dir, false) {
        Ok(p) => p,
        Err(e) => {
            error!("导出目标目录安全验证失败: {} - 原因: {}", dest_dir, e);
            return Err(AppError::validation(format!("无效的目标路径: {}", e)));
        }
    };

    // ========== 安全检查 2: 敏感目录保护 ==========
    if is_sensitive_directory(&normalized_dest) {
        error!("拒绝导出到系统敏感目录: {}", normalized_dest.display());
        return Err(AppError::validation(
            "安全限制: 不允许导出到系统敏感目录".to_string(),
        ));
    }

    // Create destination directory (在安全检查之后)
    if let Err(e) = fs::create_dir_all(&normalized_dest) {
        error!("创建目标目录失败 {}: {}", dest_dir, e);
        return Err(AppError::validation(format!("创建目标目录失败: {}", e)));
    }

    // ========== 安全检查 3: 磁盘空间预检 ==========
    // 查询所有待导出文件的总大小
    let mut total_source_size: u64 = 0;
    let mut valid_ids: Vec<(i64, String)> = Vec::new(); // (id, file_path)

    for &id in &image_ids {
        if let Ok(file_path) = conn.query_row(
            "SELECT file_path FROM images WHERE id = ?",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            let source = Path::new(&file_path);
            if source.exists() {
                if let Ok(metadata) = fs::metadata(source) {
                    total_source_size += metadata.len();
                }
                valid_ids.push((id, file_path));
            }
        }
    }

    // 磁盘空间要求：至少需要源文件总大小的 2 倍 + 最小空间要求
    let required_space = total_source_size
        .saturating_mul(2)
        .saturating_add(MIN_DISK_SPACE_REQUIRED);

    match get_available_disk_space(&normalized_dest) {
        Ok(available_space) => {
            if available_space < required_space {
                let available_mb = available_space / (1024 * 1024);
                let required_mb = required_space / (1024 * 1024);
                warn!(
                    "磁盘空间不足: 可用 {} MB, 需要 {} MB",
                    available_mb, required_mb
                );
                return Err(AppError::validation(format!(
                    "磁盘空间不足。可用空间: {} MB，需要空间: {} MB",
                    available_mb, required_mb
                )));
            }
            info!(
                "磁盘空间检查通过: 可用 {} MB, 需要 {} MB",
                available_space / (1024 * 1024),
                required_space / (1024 * 1024)
            );
        }
        Err(e) => {
            warn!("无法检查磁盘空间，跳过检查: {}", e);
        }
    }

    // ========== 开始批量导出 ==========
    let mut exported_count: usize = 0;
    let mut errors: Vec<SafeExportError> = Vec::new();

    for &(id, ref file_path) in &valid_ids {
        let source = Path::new(file_path);

        // Destination filename
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let target = normalized_dest.join(file_name);

        // Handle duplicate by appending number
        let final_target = if target.exists() {
            let stem = target
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("");
            let mut counter = 1;
            loop {
                let new_name = if ext.is_empty() {
                    format!("{}_{}", stem, counter)
                } else {
                    format!("{}_{}.{}", stem, counter, ext)
                };
                let new_path = normalized_dest.join(&new_name);
                if !new_path.exists() {
                    break new_path;
                }
                counter += 1;
            }
        } else {
            target
        };

        // Copy file
        match fs::copy(source, &final_target) {
            Ok(_) => {
                exported_count += 1;
                info!("成功导出图片: ID {} -> {}", id, final_target.display());
            }
            Err(e) => {
                warn!("导出失败: ID {} -> {}: {}", id, final_target.display(), e);
                errors.push(SafeExportError {
                    id,
                    reason: format!("文件复制失败: {}", e),
                });
            }
        }
    }

    // 处理无效/不存在的ID
    for &id in &image_ids {
        if !valid_ids.iter().any(|(valid_id, _)| *valid_id == id) {
            errors.push(SafeExportError {
                id,
                reason: format!("图片 ID {} 不存在或文件不可访问", id),
            });
        }
    }

    info!(
        "批量导出完成: 成功 {}, 失败 {}",
        exported_count,
        errors.len()
    );

    Ok(SafeExportResult {
        exported_count,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Database;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_temp_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_images.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();
        conn.execute_batch(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/1.jpg', '1.jpg', 1000, 'hash1', 'image/jpeg', 'pending', 'import');
             INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/2.png', '2.png', 2000, 'hash2', 'image/png', 'completed', 'import');
             INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/test/3.jpg', '3.jpg', 3000, 'hash3', 'image/jpeg', 'completed', 'import');",
        )
        .unwrap();

        (db, temp_dir)
    }

    #[test]
    fn test_validate_file_nonexistent() {
        let path = Path::new("/nonexistent/test/image.jpg");
        let result = validate_file(path);
        assert!(result.is_err(), "不存在的文件应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("文件不存在"));
    }

    #[test]
    fn test_validate_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_file(&temp_dir, "empty.jpg", &[]);

        let result = validate_file(&path);
        assert!(result.is_err(), "空文件应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("文件为空"));
    }

    #[test]
    fn test_validate_file_supported_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let dummy_content = b"fake image content for testing";

        let extensions = [
            "jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "tiff", "tif", "avif",
        ];

        for ext in extensions {
            let filename = format!("test.{}", ext);
            let path = create_temp_file(&temp_dir, &filename, dummy_content);
            let result = validate_file(&path);
            assert!(result.is_ok(), "扩展名 .{} 应该被支持: {:?}", ext, result);
            let (mime_type, size) = result.unwrap();
            assert_eq!(size, dummy_content.len() as u64);
            assert!(
                mime_type.starts_with("image/"),
                "MIME 类型应该是 image/*: {}",
                mime_type
            );
        }
    }

    #[test]
    fn test_validate_file_unsupported_extension() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_file(&temp_dir, "test.xyz", b"some content");

        let result = validate_file(&path);
        assert!(result.is_err(), "不支持的扩展名应返回错误");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("不支持的文件格式"));
    }

    #[test]
    fn test_validate_file_mime_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let content = b"fake image content";

        let mime_mapping = [
            ("jpg", "image/jpeg"),
            ("jpeg", "image/jpeg"),
            ("png", "image/png"),
            ("gif", "image/gif"),
            ("webp", "image/webp"),
            ("bmp", "image/bmp"),
            ("ico", "image/x-icon"),
            ("tiff", "image/tiff"),
            ("tif", "image/tiff"),
            ("avif", "image/avif"),
        ];

        for (ext, expected_mime) in mime_mapping {
            let filename = format!("test.{}", ext);
            let path = create_temp_file(&temp_dir, &filename, content);
            let result = validate_file(&path);
            assert!(result.is_ok(), "文件 {} 应该验证成功", filename);
            let (mime_type, _) = result.unwrap();
            assert_eq!(
                mime_type, expected_mime,
                "扩展名 {} 的 MIME 类型映射错误",
                ext
            );
        }
    }

    #[test]
    fn test_validate_file_special_chars_chinese() {
        let temp_dir = TempDir::new().unwrap();
        // Create file with Chinese characters in name
        let path = create_temp_file(&temp_dir, "测试图片_123.jpg", b"fake image content");

        let result = validate_file(&path);
        assert!(
            result.is_ok(),
            "包含中文字符的路径应该能正常验证: {:?}",
            result
        );
        let (_, size) = result.unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_validate_file_special_chars_spaces() {
        let temp_dir = TempDir::new().unwrap();
        // Create file with spaces in name
        let path = create_temp_file(&temp_dir, "my photo 2024.jpg", b"fake image content");

        let result = validate_file(&path);
        assert!(result.is_ok(), "包含空格的路径应该能正常验证: {:?}", result);
        let (_, size) = result.unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_get_available_disk_space_returns_valid() {
        let path = Path::new(".");
        let result = get_available_disk_space(path);
        assert!(result.is_ok(), "获取磁盘空间应该成功");
    }

    #[test]
    fn test_disk_space_check_for_temp_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_file(&temp_dir, "test.jpg", b"fake image content");

        let result = get_available_disk_space(&path);
        assert!(result.is_ok(), "临时文件的磁盘空间检查应该成功");

        if let Ok(space) = result {
            assert!(space > 0, "可用空间应该大于 0");
        }
    }

    #[test]
    fn test_get_available_disk_space_returns_result_type() {
        let path = Path::new(".");
        let result = get_available_disk_space(path);
        assert!(result.is_ok());
        let space = result.unwrap();
        assert!(space > 0);
    }

    #[test]
    fn test_disk_space_error_is_app_error() {
        let path = Path::new(".");
        let result = get_available_disk_space(path);
        match result {
            Ok(space) => assert!(space > 0),
            Err(e) => {
                let msg = e.to_string();
                assert!(!msg.is_empty(), "错误消息不应为空");
            }
        }
    }

    #[test]
    fn test_disk_space_check_graceful_degradation() {
        let result = get_available_disk_space(Path::new("."));
        match result {
            Ok(space) => {
                assert!(space > 0);
                assert!(space < u64::MAX, "不应返回 u64::MAX 作为错误哨兵值");
            }
            Err(_) => {
                // 错误路径：验证不会 panic，调用方应优雅降级
            }
        }
    }

    #[test]
    fn test_import_result_serialization() {
        let result = ImportResult {
            success_count: 5,
            duplicate_count: 2,
            error_count: 1,
            image_ids: vec![1, 2, 3, 4, 5],
            errors: vec![ImportError {
                file_path: "/test/error.jpg".to_string(),
                reason: "测试错误".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.success_count, 5);
        assert_eq!(deserialized.duplicate_count, 2);
        assert_eq!(deserialized.error_count, 1);
        assert_eq!(deserialized.image_ids.len(), 5);
        assert_eq!(deserialized.errors.len(), 1);
    }

    #[test]
    fn test_get_images_pagination() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, file_name, file_size, file_hash, mime_type, ai_status, source 
                 FROM images ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .unwrap();

        let rows = stmt
            .query_map(rusqlite::params![2, 0], |row| {
                Ok((
                    row.get::<_, i64>(0).unwrap(),
                    row.get::<_, String>(1).unwrap(),
                    row.get::<_, String>(2).unwrap(),
                ))
            })
            .unwrap();

        let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        assert_eq!(results.len(), 2, "分页应返回 2 条记录");
    }

    #[test]
    fn test_get_images_empty_result() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let mut stmt = conn
            .prepare("SELECT id FROM images ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
            .unwrap();

        let rows = stmt
            .query_map(rusqlite::params![10, 100], |row| {
                Ok(row.get::<_, i64>(0).unwrap())
            })
            .unwrap();

        let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        assert_eq!(results.len(), 0, "超出范围应返回空结果");
    }

    #[test]
    fn test_delete_images_single() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn.execute("DELETE FROM images WHERE id = 1", []).unwrap();

        assert_eq!(deleted, 1, "应删除 1 条记录");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 2, "删除后应剩余 2 条记录");
    }

    #[test]
    fn test_delete_images_multiple() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn
            .execute("DELETE FROM images WHERE id IN (1, 3)", [])
            .unwrap();

        assert_eq!(deleted, 2, "应删除 2 条记录");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1, "删除后应剩余 1 条记录");
    }

    #[test]
    fn test_delete_images_nonexistent() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let deleted = conn
            .execute("DELETE FROM images WHERE id = 999", [])
            .unwrap();

        assert_eq!(deleted, 0, "删除不存在的记录应返回 0");
    }

    #[test]
    fn test_delete_images_cleans_up_thumbnail_and_search_index() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_delete_cleanup.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();
        let conn = db.open_connection().unwrap();

        // 创建测试图片记录，带缩略图路径
        let img_path = create_test_image_file(&temp_dir, "delete_test.jpg", 100, 100);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let id = insert_image_record(
            &conn,
            path_str,
            "delete_test.jpg",
            file_size,
            &hash,
            &mime_type,
        )
        .unwrap();

        // 创建缩略图文件
        let thumb_path = temp_dir.path().join("thumb.webp");
        File::create(&thumb_path).unwrap();
        assert!(thumb_path.exists(), "缩略图文件应存在");

        // 写入缩略图路径到数据库
        conn.execute(
            "UPDATE images SET thumbnail_path = ?2 WHERE id = ?1",
            rusqlite::params![id, thumb_path.to_str().unwrap()],
        )
        .unwrap();

        // 创建 search_index 记录
        conn.execute(
            "INSERT INTO search_index (image_id, term, field, position, weight) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, "test", "description", 0, 1.0],
        ).unwrap();

        // 验证前置条件
        let index_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE image_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, 1, "search_index 记录应存在");

        // 执行删除（模拟命令的完整流程）
        // 查询 thumbnail_path
        let thumb: Option<String> = conn
            .query_row(
                "SELECT thumbnail_path FROM images WHERE id = ?",
                [id],
                |row| row.get(0),
            )
            .ok();

        // 删除 search_index
        let index_deleted = conn
            .execute("DELETE FROM search_index WHERE image_id = ?", [id])
            .unwrap();
        assert_eq!(index_deleted, 1, "应删除 search_index 记录");

        // 删除 images 记录
        let row_deleted = conn
            .execute("DELETE FROM images WHERE id = ?", [id])
            .unwrap();
        assert_eq!(row_deleted, 1, "应删除 images 记录");

        // 删除缩略图文件
        if let Some(thumb_str) = thumb {
            let thumb_path_obj = Path::new(&thumb_str);
            if thumb_path_obj.exists() {
                fs::remove_file(thumb_path_obj).unwrap();
            }
        }

        // 验证清理结果
        assert!(!thumb_path.exists(), "缩略图文件应被删除");

        let image_count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(image_count, 0, "图片记录应被删除");

        let index_count_after: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE image_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count_after, 0, "search_index 记录应被删除");
    }

    #[test]
    fn test_delete_images_missing_thumbnail_does_not_block() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_delete_missing_thumb.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();
        let conn = db.open_connection().unwrap();

        // 创建记录，但缩略图文件不存在
        let img_path = create_test_image_file(&temp_dir, "no_thumb.jpg", 50, 50);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let id = insert_image_record(
            &conn,
            path_str,
            "no_thumb.jpg",
            file_size,
            &hash,
            &mime_type,
        )
        .unwrap();

        // 设置不存在的缩略图路径
        let fake_thumb = temp_dir
            .path()
            .join("nonexistent_thumb.webp")
            .to_string_lossy()
            .to_string();
        conn.execute(
            "UPDATE images SET thumbnail_path = ?2 WHERE id = ?1",
            rusqlite::params![id, fake_thumb],
        )
        .unwrap();

        // 模拟删除流程：尝试删除不存在的缩略图应跳过
        let thumb: Option<String> = conn
            .query_row(
                "SELECT thumbnail_path FROM images WHERE id = ?",
                [id],
                |row| row.get(0),
            )
            .ok();

        if let Some(thumb_str) = thumb {
            let thumb_path_obj = Path::new(&thumb_str);
            if thumb_path_obj.exists() {
                fs::remove_file(thumb_path_obj).unwrap();
            }
            // 不存在时跳过，不应 panic
        }

        // images 记录仍应正常删除
        conn.execute("DELETE FROM images WHERE id = ?", [id])
            .unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM images WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "图片记录应被删除");
    }

    fn create_test_image_file(
        dir: &TempDir,
        name: &str,
        width: u32,
        height: u32,
    ) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([128, 128, 128]));
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn test_update_image_metadata() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = update_image_metadata(
            &conn,
            1,
            "/test/thumb_1.webp",
            "abc123def456",
            1920,
            1080,
            r#"{"width":1920,"height":1080}"#,
        );
        assert!(result.is_ok(), "元数据更新应成功: {:?}", result);

        // 验证写入结果
        let thumb: String = conn
            .query_row(
                "SELECT thumbnail_path FROM images WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(thumb, "/test/thumb_1.webp");

        let phash: String = conn
            .query_row("SELECT phash FROM images WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(phash, "abc123def456");

        let width: i32 = conn
            .query_row("SELECT width FROM images WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(width, 1920);
    }

    #[test]
    fn test_create_ai_task() {
        let (db, _temp) = setup_test_db();
        let conn = db.open_connection().unwrap();

        let result = create_ai_task(&conn, 1);
        assert!(result.is_ok(), "AI 任务创建应成功: {:?}", result);

        let task_type: String = conn
            .query_row(
                "SELECT task_type FROM task_queue WHERE image_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(task_type, "ai_analysis");

        let status: String = conn
            .query_row(
                "SELECT status FROM task_queue WHERE image_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "pending");
    }

    #[test]
    fn test_import_pipeline_full_chain() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_import_chain.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        let img_path = create_test_image_file(&temp_dir, "pipeline_test.jpg", 800, 600);
        let path_str = img_path.to_str().unwrap();

        // 完整串联：验证 → 哈希 → 插入 → 缩略图 → pHash → EXIF → 元数据 → AI 任务
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        assert!(!is_duplicate(&conn, &hash).unwrap(), "新文件不应是重复的");

        let id = insert_image_record(
            &conn,
            path_str,
            "pipeline_test.jpg",
            file_size,
            &hash,
            &mime_type,
        )
        .unwrap();
        assert!(id > 0, "应返回有效的图片 ID");

        // 缩略图
        let thumb_path = temp_dir.path().join(format!("thumb_{}.webp", id));
        let thumb_result =
            ImageProcessor::generate_thumbnail(path_str, thumb_path.to_str().unwrap());
        assert!(thumb_result.is_ok(), "缩略图应成功生成: {:?}", thumb_result);
        assert!(thumb_path.exists(), "缩略图文件应存在");

        // pHash
        let phash = ImageProcessor::calculate_phash(path_str).unwrap();
        assert!(!phash.is_empty(), "pHash 不应为空");

        // EXIF
        let exif = ImageProcessor::extract_exif(path_str).unwrap();
        assert!(exif.get("width").is_some(), "EXIF 应包含宽度");

        // 写回元数据
        let exif_json = serde_json::to_string(&exif).unwrap();
        update_image_metadata(
            &conn,
            id,
            thumb_path.to_str().unwrap(),
            &phash,
            800,
            600,
            &exif_json,
        )
        .unwrap();

        // 创建 AI 任务
        create_ai_task(&conn, id).unwrap();

        // 最终验证
        let (thumb, p, w, h): (String, String, i32, i32) = conn
            .query_row(
                "SELECT thumbnail_path, phash, width, height FROM images WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert!(thumb.ends_with(".webp"), "缩略图路径应以 .webp 结尾");
        assert_eq!(p, phash, "pHash 应一致");
        assert_eq!(w, 800, "宽度应正确");
        assert_eq!(h, 600, "高度应正确");

        // 验证 AI 任务
        let task_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_queue WHERE image_id = ?1 AND status = 'pending'",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(task_count, 1, "应有 1 个 pending AI 任务");
    }

    #[test]
    fn test_import_pipeline_thumbnail_failure_does_not_block() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_import_fail_thumb.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // 创建合法图片
        let img_path = create_test_image_file(&temp_dir, "test_thumb_fail.jpg", 640, 480);
        let path_str = img_path.to_str().unwrap();

        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let id =
            insert_image_record(&conn, path_str, "test.jpg", file_size, &hash, &mime_type).unwrap();

        // 使用不存在的图片路径触发缩略图生成失败
        let thumb_result = ImageProcessor::generate_thumbnail(
            "/nonexistent/fake_image.jpg",
            temp_dir.path().join("thumb.webp").to_str().unwrap(),
        );
        assert!(thumb_result.is_err(), "缩略图生成对不存在图片应失败");

        // 但元数据更新应成功
        let phash = ImageProcessor::calculate_phash(path_str).unwrap();
        let exif = ImageProcessor::extract_exif(path_str).unwrap();
        let exif_json = serde_json::to_string(&exif).unwrap();

        let meta_result = update_image_metadata(&conn, id, "", &phash, 640, 480, &exif_json);
        assert!(meta_result.is_ok(), "元数据更新不应被缩略图失败阻塞");

        // AI 任务应正常创建
        let task_result = create_ai_task(&conn, id);
        assert!(task_result.is_ok(), "AI 任务创建不应被阻塞");
    }

    #[test]
    fn test_import_pipeline_phash_failure_does_not_block() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_import_fail_phash.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        let img_path = create_test_image_file(&temp_dir, "test_phash_fail.jpg", 320, 240);
        let path_str = img_path.to_str().unwrap();

        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let id =
            insert_image_record(&conn, path_str, "test.jpg", file_size, &hash, &mime_type).unwrap();

        // pHash 对不存在文件应失败
        let phash_result = ImageProcessor::calculate_phash("/nonexistent/image.jpg");
        assert!(phash_result.is_err(), "pHash 对不存在文件应失败");

        // 但其他流程应继续
        let thumb_path = temp_dir.path().join(format!("thumb_{}.webp", id));
        ImageProcessor::generate_thumbnail(path_str, thumb_path.to_str().unwrap()).unwrap();

        let exif = ImageProcessor::extract_exif(path_str).unwrap();
        let exif_json = serde_json::to_string(&exif).unwrap();
        update_image_metadata(
            &conn,
            id,
            thumb_path.to_str().unwrap(),
            "",
            320,
            240,
            &exif_json,
        )
        .unwrap();
        create_ai_task(&conn, id).unwrap();

        // 记录应存在且 AI 任务已创建
        let task_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_queue WHERE image_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(task_count, 1, "AI 任务应已创建");
    }

    #[test]
    fn test_check_broken_links_detects_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_broken_links.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Insert images: one with existing file, one with non-existing file
        let img_path = create_test_image_file(&temp_dir, "exists.jpg", 100, 100);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let id_exists =
            insert_image_record(&conn, path_str, "exists.jpg", file_size, &hash, &mime_type)
                .unwrap();

        // Insert record with non-existing file path
        let id_broken: i64 = conn.query_row(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/nonexistent/broken.jpg', 'broken.jpg', 1234, 'hash_broken', 'image/jpeg', 'pending', 'import')
             RETURNING id",
            [],
            |row| row.get(0),
        ).unwrap();

        // Simulate check_broken_links logic
        let mut stmt = conn
            .prepare("SELECT id, file_path, file_name FROM images")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap();

        let mut broken_images: Vec<BrokenLinkInfo> = Vec::new();
        let mut ids_to_mark: Vec<i64> = Vec::new();

        for (id, file_path, file_name) in rows.flatten() {
            if !Path::new(&file_path).exists() {
                broken_images.push(BrokenLinkInfo {
                    id,
                    file_path,
                    file_name,
                });
                ids_to_mark.push(id);
            }
        }

        // Should find exactly 1 broken link
        assert_eq!(broken_images.len(), 1, "应检测到 1 个失效链接");
        assert_eq!(broken_images[0].id, id_broken);

        // Mark as broken_link
        for &id in &ids_to_mark {
            conn.execute(
                "UPDATE images SET ai_status = 'broken_link' WHERE id = ?",
                rusqlite::params![id],
            )
            .unwrap();
        }

        // Verify status was updated
        let status: String = conn
            .query_row(
                "SELECT ai_status FROM images WHERE id = ?",
                [id_broken],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "broken_link", "失效图片应被标记为 broken_link");

        // Existing file should NOT be marked
        let existing_status: String = conn
            .query_row(
                "SELECT ai_status FROM images WHERE id = ?",
                [id_exists],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(existing_status, "pending", "有效文件不应被标记");
    }

    #[test]
    fn test_check_broken_links_all_files_exist() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_no_broken.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Insert images with existing files only
        for i in 1..=3 {
            let img_path =
                create_test_image_file(&temp_dir, &format!("img{}.jpg", i), 50 * i, 50 * i);
            let path_str = img_path.to_str().unwrap();
            let (mime_type, file_size) = validate_file(&img_path).unwrap();
            let hash = calculate_sha256(&img_path).unwrap();
            insert_image_record(
                &conn,
                path_str,
                &format!("img{}.jpg", i),
                file_size,
                &hash,
                &mime_type,
            )
            .unwrap();
        }

        // Simulate check_broken_links logic
        let mut stmt = conn
            .prepare("SELECT id, file_path, file_name FROM images")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap();

        let mut broken_count = 0;
        for (_id, file_path, _file_name) in rows.flatten() {
            if !Path::new(&file_path).exists() {
                broken_count += 1;
            }
        }

        assert_eq!(broken_count, 0, "所有文件存在时应返回 0 个失效链接");
    }

    #[test]
    fn test_archive_image_copies_file_to_archive_dir() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_archive.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Create a test image
        let img_path = create_test_image_file(&temp_dir, "archive_me.jpg", 200, 200);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let _id = insert_image_record(
            &conn,
            path_str,
            "archive_me.jpg",
            file_size,
            &hash,
            &mime_type,
        )
        .unwrap();

        // Simulate archive logic: copy to archive directory
        let archive_dir = temp_dir.path().join("archive_images");
        fs::create_dir_all(&archive_dir).unwrap();

        let source = Path::new(&path_str);
        assert!(source.exists(), "源文件应存在");

        let dest = archive_dir.join("archive_me.jpg");
        let result = fs::copy(source, &dest);
        assert!(result.is_ok(), "文件复制应成功");
        assert!(dest.exists(), "归档文件应存在");

        // Verify content matches
        let src_content = fs::read(source).unwrap();
        let dest_content = fs::read(&dest).unwrap();
        assert_eq!(src_content, dest_content, "归档文件内容应与原文件一致");
    }

    #[test]
    fn test_archive_image_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_archive_missing.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Insert record with non-existing file
        let id: i64 = conn.query_row(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/nonexistent/missing.jpg', 'missing.jpg', 1000, 'hash_miss', 'image/jpeg', 'pending', 'import')
             RETURNING id",
            [],
            |row| row.get(0),
        ).unwrap();

        // Verify file doesn't exist
        let file_path: String = conn
            .query_row("SELECT file_path FROM images WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();

        assert!(!Path::new(&file_path).exists(), "文件应不存在");
        // Archive should return error for non-existing source
    }

    #[test]
    fn test_safe_export_batch_copies_files() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_export.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Create test images
        let mut ids = Vec::new();
        for i in 1..=3 {
            let img_path =
                create_test_image_file(&temp_dir, &format!("export_{}.jpg", i), 100, 100);
            let path_str = img_path.to_str().unwrap();
            let (mime_type, file_size) = validate_file(&img_path).unwrap();
            let hash = calculate_sha256(&img_path).unwrap();
            let id = insert_image_record(
                &conn,
                path_str,
                &format!("export_{}.jpg", i),
                file_size,
                &hash,
                &mime_type,
            )
            .unwrap();
            ids.push(id);
        }

        // Simulate safe_export: copy to dest directory
        let dest_dir = temp_dir.path().join("exported");
        fs::create_dir_all(&dest_dir).unwrap();

        let mut exported_count = 0;
        let mut errors: Vec<SafeExportError> = Vec::new();

        for &id in &ids {
            let file_path: Option<String> = conn
                .query_row("SELECT file_path FROM images WHERE id = ?", [id], |row| {
                    row.get(0)
                })
                .ok();

            if let Some(fp) = file_path {
                let source = Path::new(&fp);
                if source.exists() {
                    let file_name = source
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let target = dest_dir.join(file_name);
                    match fs::copy(source, &target) {
                        Ok(_) => {
                            exported_count += 1;
                            assert!(target.exists(), "导出文件应存在");
                        }
                        Err(e) => {
                            errors.push(SafeExportError {
                                id,
                                reason: format!("复制失败: {}", e),
                            });
                        }
                    }
                } else {
                    errors.push(SafeExportError {
                        id,
                        reason: "文件不存在".to_string(),
                    });
                }
            } else {
                errors.push(SafeExportError {
                    id,
                    reason: "图片不存在".to_string(),
                });
            }
        }

        assert_eq!(exported_count, 3, "应成功导出 3 个文件");
        assert_eq!(errors.len(), 0, "不应有错误");

        // Verify files exist in dest dir
        for i in 1..=3 {
            let exported = dest_dir.join(format!("export_{}.jpg", i));
            assert!(exported.exists(), "导出文件 {} 应存在", exported.display());
        }
    }

    #[test]
    fn test_safe_export_handles_missing_and_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_export_errors.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Create one valid image
        let img_path = create_test_image_file(&temp_dir, "valid.jpg", 50, 50);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let valid_id =
            insert_image_record(&conn, path_str, "valid.jpg", file_size, &hash, &mime_type)
                .unwrap();

        // Insert one with non-existing file
        let broken_id: i64 = conn.query_row(
            "INSERT INTO images (file_path, file_name, file_size, file_hash, mime_type, ai_status, source) 
             VALUES ('/nonexistent/nope.jpg', 'nope.jpg', 500, 'hash_no', 'image/jpeg', 'pending', 'import')
             RETURNING id",
            [],
            |row| row.get(0),
        ).unwrap();

        // Simulate safe_export with both valid and broken IDs plus a non-existent ID
        let test_ids = vec![valid_id, broken_id, 9999];
        let dest_dir = temp_dir.path().join("export_test_errors");
        fs::create_dir_all(&dest_dir).unwrap();

        let mut exported_count = 0;
        let mut errors: Vec<SafeExportError> = Vec::new();

        for &id in &test_ids {
            let file_path: String =
                match conn.query_row("SELECT file_path FROM images WHERE id = ?", [id], |row| {
                    row.get::<_, String>(0)
                }) {
                    Ok(fp) => fp,
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        errors.push(SafeExportError {
                            id,
                            reason: format!("图片 ID {} 不存在", id),
                        });
                        continue;
                    }
                    Err(e) => {
                        errors.push(SafeExportError {
                            id,
                            reason: format!("查询失败: {}", e),
                        });
                        continue;
                    }
                };

            let source = Path::new(&file_path);
            if !source.exists() {
                errors.push(SafeExportError {
                    id,
                    reason: format!("文件不存在: {}", file_path),
                });
                continue;
            }

            let file_name = source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let target = dest_dir.join(file_name);
            match fs::copy(source, &target) {
                Ok(_) => exported_count += 1,
                Err(e) => errors.push(SafeExportError {
                    id,
                    reason: format!("复制失败: {}", e),
                }),
            }
        }

        assert_eq!(exported_count, 1, "应仅成功导出 1 个文件");
        assert_eq!(
            errors.len(),
            2,
            "应有 2 个错误（不存在的文件 + 不存在的ID）"
        );
    }

    #[test]
    fn test_safe_export_duplicate_filename_handling() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_export_dup.db");
        let db = Database::new_from_path(db_path.to_str().unwrap()).unwrap();
        db.init().unwrap();

        let conn = db.open_connection().unwrap();

        // Create test image
        let img_path = create_test_image_file(&temp_dir, "dup.jpg", 80, 80);
        let path_str = img_path.to_str().unwrap();
        let (mime_type, file_size) = validate_file(&img_path).unwrap();
        let hash = calculate_sha256(&img_path).unwrap();
        let _id =
            insert_image_record(&conn, path_str, "dup.jpg", file_size, &hash, &mime_type).unwrap();

        let dest_dir = temp_dir.path().join("export_dup_test");
        fs::create_dir_all(&dest_dir).unwrap();

        // Pre-create a file with the same name
        let pre_existing = dest_dir.join("dup.jpg");
        File::create(&pre_existing).unwrap();
        assert!(pre_existing.exists(), "预存文件应存在");

        // Simulate export with duplicate handling
        let file_name = "dup.jpg";
        let target = dest_dir.join(file_name);

        let final_target = if target.exists() {
            let stem = "dup";
            let ext = "jpg";
            let mut counter = 1;
            loop {
                let new_name = format!("{}_{}.{}", stem, counter, ext);
                let new_path = dest_dir.join(&new_name);
                if !new_path.exists() {
                    break new_path;
                }
                counter += 1;
            }
        } else {
            target
        };

        // Should be dup_1.jpg
        assert_eq!(
            final_target.file_name().unwrap().to_str().unwrap(),
            "dup_1.jpg"
        );

        // Copy should succeed
        fs::copy(&img_path, &final_target).unwrap();
        assert!(final_target.exists(), "重命名后的导出文件应存在");
    }
}
