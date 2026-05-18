#![allow(missing_docs)]
use std::path::{Component, Path, PathBuf};

use crate::utils::error::AppError;

use tracing::{info, warn};

/// 验证备份/恢复路径的安全性
///
/// 专为 backup/restore 场景设计：
/// - **backup**: 用户选择保存位置 → 允许任意绝对路径但要验证安全性
/// - **restore**: 用户选择备份文件 → 验证文件存在且路径安全
///
/// # 安全机制
/// 1. 空路径拒绝
/// 2. 路径长度限制 (240 字符)
/// 3. UNC 路径拦截
/// 4. Windows 设备路径拦截
/// 5. NTFS 数据流拦截
/// 6. 组件级 `..` 遍历检查
/// 7. 编码遍历序列检测
/// 8. 控制字符检测
/// 9. Windows 保留名称检测
///
/// # 参数
/// - `path`: 用户输入的路径字符串
/// - `must_exist`: 路径是否必须已存在 (restore=true, backup=false)
pub(crate) fn validate_backup_restore_path(path: &str, must_exist: bool) -> Result<PathBuf, AppError> {
    let input = path.trim();

    // ===== 层1: 基础输入验证 =====
    if input.is_empty() {
        return Err(AppError::validation("路径不能为空"));
    }

    // 路径长度限制（Windows MAX_PATH = 260，留余量）
    if input.len() > 240 {
        return Err(AppError::validation(format!(
            "路径过长 ({} 字符，最大 240)",
            input.len()
        )));
    }

    let path_buf = PathBuf::from(input);

    // ===== 层2: 特殊路径模式拦截 =====
    let input_lower = input.to_lowercase();

    // Windows UNC 路径拦截 (\\server\share, //server/share)
    if input.starts_with("\\\\") || input.starts_with("//") {
        return Err(AppError::validation("不允许使用 UNC 路径"));
    }

    // Windows 设备路径拦截 (\\.\COM1, \\?\C:, etc.)
    if input_lower.contains("\\\\.\\") || input_lower.contains("\\\\?\\") {
        return Err(AppError::validation("不允许使用设备路径"));
    }

    // NTFS Alternate Data Streams 拦截 (file.txt::$DATA)
    if input.contains("::$") {
        return Err(AppError::validation("检测到 NTFS 数据流攻击"));
    }

    // ===== 层3: Windows 保留名称检测 =====
    #[cfg(windows)]
    {
        const RESERVED_NAMES: &[&str] = &[
            "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
            "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
        ];
        let file_name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if RESERVED_NAMES.contains(&file_name.as_str()) {
            return Err(AppError::validation(format!(
                "使用 Windows 保留文件名: {}",
                file_name
            )));
        }
    }

    // ===== 层4: 组件级遍历检查 =====
    let has_parent_dir = path_buf
        .components()
        .any(|c| matches!(c, Component::ParentDir));
    if has_parent_dir {
        warn!("备份/恢复路径遍历检测: {}", input);
        return Err(AppError::validation("路径不允许包含 '..' 目录遍历序列"));
    }

    // 编码后的遍历序列拦截
    if contains_encoded_traversal(input) {
        return Err(AppError::validation("检测到编码后的路径遍历尝试"));
    }

    // 控制字符和空字节检查
    for ch in input.chars() {
        if ch.is_control() && ch != '\t' {
            return Err(AppError::validation("路径包含非法控制字符"));
        }
    }

    // ===== 层5: 存在性验证 + 规范化 =====
    if must_exist {
        // restore 场景：文件必须存在
        match path_buf.canonicalize() {
            Ok(canonical) => {
                info!("恢复路径验证通过: '{}' -> '{}'", input, canonical.display());
                Ok(canonical)
            }
            Err(e) => {
                warn!("恢复路径不存在或无法访问: {} - {}", input, e);
                Err(AppError::validation(format!(
                    "备份文件不存在或无法访问: {}",
                    input
                )))
            }
        }
    } else {
        // backup 场景：目标文件可能尚不存在，验证父目录可写性
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                // 如果父目录存在，检查是否可写
                if parent.exists() {
                    // 验证父目录不是敏感系统目录
                    if is_system_sensitive_directory(parent) {
                        return Err(AppError::validation("安全限制: 不允许备份到系统敏感目录"));
                    }
                    info!("备份路径验证通过: '{}'", input);
                    Ok(path_buf)
                } else {
                    // 父目录不存在时，尝试规范化路径以捕获潜在问题
                    let normalized = normalize_path_for_backup(&path_buf)?;
                    info!(
                        "备份路径验证通过(新建): '{}' -> '{}'",
                        input,
                        normalized.display()
                    );
                    Ok(normalized)
                }
            } else {
                // 相对路径无父目录组件（如 "file.zip"），直接使用
                info!("备份路径验证通过(相对路径): '{}'", input);
                Ok(path_buf)
            }
        } else {
            Ok(path_buf)
        }
    }
}

/// 检查输入是否包含编码后的路径遍历模式
fn contains_encoded_traversal(input: &str) -> bool {
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
    ];

    let input_lower = input.to_lowercase();
    encoded_patterns
        .iter()
        .any(|p| input_lower.contains(&p.to_lowercase()))
}

/// 检查是否为系统敏感目录（禁止作为备份目标）
#[cfg(windows)]
fn is_system_sensitive_directory(path: &Path) -> bool {
    const SENSITIVE_DIRS: &[&str] = &[
        r"C:\Windows",
        r"C:\Program Files",
        r"C:\Program Files (x86)",
        r"C:\ProgramData",
        r"C:\System32",
    ];

    let path_str = path.to_string_lossy().to_lowercase();
    SENSITIVE_DIRS.iter().any(|sensitive| {
        path_str.starts_with(&sensitive.to_lowercase()) || *sensitive == path.to_string_lossy()
    })
}

#[cfg(not(windows))]
fn is_system_sensitive_directory(path: &Path) -> bool {
    const SENSITIVE_DIRS: &[&str] = &["/usr/bin", "/usr/sbin", "/bin", "/sbin", "/etc"];
    SENSITIVE_DIRS
        .iter()
        .any(|sensitive| path.starts_with(sensitive) || path == Path::new(sensitive))
}

/// 标准化备份目标路径（不要求文件存在）
/// 用于处理不存在的目标路径（如新建备份文件）
fn normalize_path_for_backup(path: &Path) -> Result<PathBuf, AppError> {
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
                if normalized.pop() {
                    parent_count -= 1;
                } else {
                    return Err(AppError::validation("路径解析结果超出根目录边界"));
                }
            }
            Component::CurDir => {
                continue;
            }
        }
    }

    if parent_count.abs() > 10 {
        return Err(AppError::validation("路径包含过多的目录层级跳转"));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_settings_sec_001_validate_empty_path() {
        let result = validate_backup_restore_path("", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("不能为空"), "error: {}", err);
    }

    #[test]
    fn tc_settings_sec_002_validate_whitespace_only_path() {
        let result = validate_backup_restore_path("   ", false);
        assert!(result.is_err());
    }

    #[test]
    fn tc_settings_sec_003_validate_path_traversal() {
        let malicious_paths = vec![
            "../../etc/passwd",
            "..\\..\\windows\\system32\\config",
            "/home/user/../../../etc/shadow",
            "C:\\Users\\..\\..\\Windows\\System32",
            "folder/../../secret",
        ];
        for path in malicious_paths {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject: {}", path);
        }
    }

    #[test]
    fn tc_settings_sec_004_validate_unc_path() {
        let unc_paths = vec![
            "\\\\server\\share\\backup.zip",
            "//server/share/backup.zip",
            "\\\\192.168.1.100\\share\\file.enc",
        ];
        for path in unc_paths {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject UNC: {}", path);
            let err = result.unwrap_err().to_string();
            assert!(err.contains("UNC"), "error should mention UNC: {}", err);
        }
    }

    #[test]
    fn tc_settings_sec_005_validate_device_path() {
        let device_paths = vec![
            "\\\\.\\COM1",
            "\\\\?\\C:\\test.zip",
            "\\\\.\\PhysicalDrive0",
        ];
        for path in device_paths {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject device path: {}", path);
        }
    }

    #[test]
    fn tc_settings_sec_006_validate_ntfs_alternate_data_stream() {
        let ads_paths = vec![
            "C:\\test.zip::$DATA",
            "backup.zip::$ALTERNATE",
            "file.txt::$QUOTA",
        ];
        for path in ads_paths {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject NTFS stream: {}", path);
            let err = result.unwrap_err().to_string();
            assert!(err.contains("NTFS") || err.contains("数据流"), "error: {}", err);
        }
    }

    #[test]
    fn tc_settings_sec_007_validate_encoded_traversal() {
        let encoded_paths = vec![
            "%2e%2e/%2e%2e/etc/passwd",
            "%252e%252e",
            "..%2f..%2fetc%2fpasswd",
            "%c0%ae%c0%ae/etc",
        ];
        for path in encoded_paths {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject encoded traversal: {}", path);
        }
    }

    #[test]
    fn tc_settings_sec_009_validate_long_path() {
        let long_path = format!("C:\\{}", "a".repeat(250));
        let result = validate_backup_restore_path(&long_path, false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("过长"), "error: {}", err);
    }

    #[test]
    fn tc_settings_sec_010_validate_valid_backup_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let valid_paths = vec![
            temp_dir.path().join("backup.zip").to_string_lossy().to_string(),
            temp_dir.path().join("subdir").join("data.enc").to_string_lossy().to_string(),
        ];
        for path in valid_paths {
            let result = validate_backup_restore_path(&path, false);
            assert!(result.is_ok(), "should accept: {}", path);
        }
    }

    #[test]
    fn tc_settings_sec_011_validate_existing_restore_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let backup_file = temp_dir.path().join("existing_backup.zip");
        std::fs::write(&backup_file, b"fake backup data").unwrap();
        let result = validate_backup_restore_path(backup_file.to_string_lossy().as_ref(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn tc_settings_sec_012_validate_nonexistent_restore_path() {
        let result = validate_backup_restore_path("C:\\nonexistent\\path\\backup_99999.zip", true);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("不存在") || err.contains("无法访问"), "error: {}", err);
    }

    #[cfg(windows)]
    #[test]
    fn tc_settings_sec_013_validate_windows_reserved_names() {
        let reserved_names = vec![
            "C:\\test\\CON.zip",
            "C:\\test\\PRN.enc",
            "C:\\test\\AUX.zip",
            "C:\\test\\NUL.enc",
            "C:\\test\\COM1.zip",
            "C:\\test\\LPT9.enc",
        ];
        for path in reserved_names {
            let result = validate_backup_restore_path(path, false);
            assert!(result.is_err(), "should reject reserved name: {}", path);
        }
    }

    #[test]
    fn tc_settings_sec_015_accept_normal_user_paths() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let normal_paths = vec![
            temp_dir.path().join("my_backup.zip").to_string_lossy().to_string(),
            temp_dir.path().join("backup-with-dashes.zip").to_string_lossy().to_string(),
            temp_dir.path().join("backup.with.dots.zip").to_string_lossy().to_string(),
        ];
        for path in normal_paths {
            let result = validate_backup_restore_path(&path, false);
            assert!(result.is_ok(), "should accept: {}", path);
        }
    }
}
