$file = "e:\ArcaneCodex\src-tauri\src\commands\settings.rs"
$content = [System.IO.File]::ReadAllText($file, [System.Text.UTF8Encoding]::new($false))

# Line 33: info!("保存配置: {} = {}", ...)
$content = $content -replace 'info!\(".*?", config\.key, config\.value\);', 'info!("Config saved: {} = {}", config.key, config.value);'

# Line 57: info!("设置配置: {} = {}", ...)
$content = $content -replace 'info!\(".*?\{\} = \{\}", key, value\);', 'info!("Config set: {} = {}", key, value);'

# Line 85: info!("获取所有配置: {} 条", ...)
$content = $content -replace 'info!\(".*?", result\.len\(\)\);', 'info!("All configs retrieved: {} items", result.len());'

# Line 110: AppError::config(format!("无法读取配置: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read config: {}", e))'

# Line 121: return Err(AppError::config("配置键不能为空".to_string()))
$content = $content -replace 'return Err\(AppError::config\(".*?"\.to_string\(\)\)\)', 'return Err(AppError::config("Config key cannot be empty".to_string()))'

# Line 126: AppError::config(format!("无法保存配置: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to save config: {}", e))'

# Line 137: .map_err(|e| AppError::config(format!("无法创建 zip: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create zip: {}", e)))?'

# Line 140: AppError::config(format!("无法添加文件到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add file to zip: {}", e))'

# Line 145: AppError::config(format!("无法写入 zip 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write zip data: {}", e))'

# Line 149: AppError::config(format!("无法完成 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize zip: {}", e))'

# Line 161: .map_err(|e| AppError::config(format!("无法创建 WAL: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create WAL: {}", e)))?'

# Line 164: AppError::config(format!("无法添加 WAL 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add WAL to zip: {}", e))'

# Line 169: AppError::config(format!("无法写入 WAL 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write WAL data: {}", e))'

# Line 173: AppError::config(format!("无法完成 WAL zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize WAL zip: {}", e))'

# Line 185: .map_err(|e| AppError::config(format!("无法创建 SHM: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create SHM: {}", e)))?'

# Line 188: AppError::config(format!("无法添加 SHM 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add SHM to zip: {}", e))'

# Line 193: AppError::config(format!("无法写入 SHM 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write SHM data: {}", e))'

# Line 197: AppError::config(format!("无法完成 SHM zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize SHM zip: {}", e))'

# Line 202: AppError::config(format!("无法创建 zip 写入器: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create zip writer: {}", e))'

# Line 208: .map_err(|e| AppError::config(format!("无法序列化数据库: {}", e)))??;
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?\?', '.map_err(|e| AppError::config(format!("Failed to serialize database: {}", e)))?'

# Line 210: info!("数据库备份完成: {}", result)
$content = $content -replace 'info!\(".*?: \{\}", result\);', 'info!("Database backup completed: {}", result);'

# Line 230: return Err(AppError::config("备份文件路径不能为空".to_string()))
$content = $content -replace 'return Err\(AppError::config\(".*?"\.to_string\(\)\)\)', 'return Err(AppError::config("Backup file path cannot be empty".to_string()))'

# Line 235: AppError::config(format!("无法读取备份文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read backup file: {}", e))'

# Line 239: AppError::config(format!("无法打开 ZIP: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?ZIP.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to open ZIP: {}", e))'

# Line 251: AppError::config(format!("无法读取备份数据库: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read backup database: {}", e))'

# Line 257: AppError::config(format!("无法解压 ZIP: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?ZIP.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to extract ZIP: {}", e))'

# Line 264: warn!("Zip Slip 攻击检测: {}", entry_name)
$content = $content -replace 'warn!\(".*?ZIP.*?: \{\}", entry_name\);', 'warn!("Zip Slip attack detected: {}", entry_name);'

# Line 272: warn!("Zip Slip 已阻止: {} ...", entry_name)
$content = $content -replace 'warn!\("Zip Slip.*?: \{\}.*?", entry_name\);', 'warn!("Zip Slip blocked: {}", entry_name);'

# Line 279: AppError::config(format!("无法创建临时目录: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create temp directory: {}", e))'

# Line 285: AppError::config(format!("无法创建输出文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create output file: {}", e))'

# Line 290: AppError::config(format!("无法写入文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write file: {}", e))'

# Line 294: AppError::config(format!("无法读取 ZIP 条目: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read ZIP entry: {}", e))'

# Line 297: AppError::config(format!("无法解压条目: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to extract entry: {}", e))'

# Line 312: format!("备份数据库版本 (v{}) 高于当前版本 (v{})...", backup_version, current_version)
$content = $content -replace 'format!\(".*?\(v\{\}\).*?\(v\{\}\).*?", backup_version, current_version\)', 'format!("Backup DB version (v{}) is higher than current version (v{}). Please upgrade the app first.", backup_version, current_version)'

# Line 319: .map_err(|e| AppError::config(format!("无法读取备份版本: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to read backup version: {}", e)))?'

# Line 321: .map_err(|e| AppError::config(format!("无法读取当前版本: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to read current version: {}", e)))?'

# Line 327: .map_err(|e| AppError::config(format!("无法创建备份目录: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create backup directory: {}", e)))?'

# Line 329: .map_err(|e| AppError::config(format!("无法复制数据库文件: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to copy database file: {}", e)))?'

# Line 338: format!("备份数据库版本 (v{}) 高于当前版本 (v{})...", ...)
$content = $content -replace 'format!\(".*?\(v\{\}\).*?\(v\{\}\).*?", backup_version, current_version\)', 'format!("Backup DB version (v{}) is higher than current version (v{}). Please upgrade the app first.", backup_version, current_version)'

# Line 343: info!("数据库恢复完成: v{} -> v{}", ...)
$content = $content -replace 'info!\(".*?v\{\}.*?v\{\}", backup_version, current_version\);', 'info!("Database restore completed: v{} -> v{}", backup_version, current_version);'

# Line 349: AppError::config(format!("无法删除旧数据库: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to delete old database: {}", e))'

# Line 355: AppError::config(format!("无法添加数据库到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add database to zip: {}", e))'

# Line 364: AppError::config(format!("无法添加 WAL 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add WAL to zip: {}", e))'

# Line 377: AppError::config(format!("无法添加 SHM 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add SHM to zip: {}", e))'

# Line 390: .map_err(|e| AppError::config(format!("无法序列化数据库: {}", e)))??;
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?: \{\}", e\)\)\)\?\?', '.map_err(|e| AppError::config(format!("Failed to serialize database: {}", e)))?'

# Line 392: info!("备份已保存到: {}", backup_path)
$content = $content -replace 'info!\(".*?: \{\}", backup_path\);', 'info!("Backup saved to: {}", backup_path);'

# Line 538: assert!(count > 0, "...")
$content = $content -replace 'assert!\(count > 0, ".*?"\);', 'assert!(count > 0, "Should have at least one config after insert");'

# Line 553: return Err(AppError::config("配置键不能为空"...))
$content = $content -replace 'return Err\(AppError::config\(".*?"\.to_string\(\)\)\)', 'return Err(AppError::config("Config key cannot be empty".to_string()))'

# Line 565: AppError::config(format!("无法读取配置: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read config: {}", e))'

# Line 570: AppError::config(format!("无法保存配置: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to save config: {}", e))'

# Line 581: .map_err(|e| AppError::config(format!("无法创建 zip: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create zip: {}", e)))?'

# Line 584: AppError::config(format!("无法添加文件到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add file to zip: {}", e))'

# Line 588: AppError::config(format!("无法写入 zip 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write zip data: {}", e))'

# Line 591: AppError::config(format!("无法完成 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize zip: {}", e))'

# Line 602: .map_err(|e| AppError::config(format!("无法创建 WAL: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create WAL: {}", e)))?'

# Line 604: AppError::config(format!("无法添加 WAL 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add WAL to zip: {}", e))'

# Line 608: AppError::config(format!("无法写入 WAL 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write WAL data: {}", e))'

# Line 611: AppError::config(format!("无法完成 WAL zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?WAL.*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize WAL zip: {}", e))'

# Line 623: .map_err(|e| AppError::config(format!("无法创建 SHM: {}", e)))?
$content = $content -replace '\.map_err\(\|e\| AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)\)\?', '.map_err(|e| AppError::config(format!("Failed to create SHM: {}", e)))?'

# Line 625: AppError::config(format!("无法添加 SHM 到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add SHM to zip: {}", e))'

# Line 629: AppError::config(format!("无法写入 SHM 数据: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write SHM data: {}", e))'

# Line 632: AppError::config(format!("无法完成 SHM zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?SHM.*?zip: \{\}", e\)\)', 'AppError::config(format!("Failed to finalize SHM zip: {}", e))'

# Line 637: AppError::config(format!("无法创建 zip 写入器: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?zip.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create zip writer: {}", e))'

# Line 652: return Err(AppError::config("备份文件路径不能为空"...))
$content = $content -replace 'return Err\(AppError::config\(".*?"\.to_string\(\)\)\)', 'return Err(AppError::config("Backup file path cannot be empty".to_string()))'

# Line 656: AppError::config(format!("无法读取备份文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read backup file: {}", e))'

# Line 659: AppError::config(format!("无法打开 ZIP: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?ZIP.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to open ZIP: {}", e))'

# Line 671: AppError::config(format!("无法读取备份数据库: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read backup database: {}", e))'

# Line 676: AppError::config(format!("无法解压 ZIP: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?ZIP.*?: \{\}", e\)\)', 'AppError::config(format!("Failed to extract ZIP: {}", e))'

# Line 682: AppError::config(format!("无法创建临时目录: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create temp directory: {}", e))'

# Line 688: AppError::config(format!("无法创建输出文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to create output file: {}", e))'

# Line 693: AppError::config(format!("无法写入文件: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to write file: {}", e))'

# Line 697: AppError::config(format!("无法读取 ZIP 条目: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to read ZIP entry: {}", e))'

# Line 700: AppError::config(format!("无法解压条目: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to extract entry: {}", e))'

# Line 714: format!("备份数据库版本 (v{}) 高于当前版本 (v{})...", ...)
$content = $content -replace 'format!\(".*?\(v\{\}\).*?\(v\{\}\).*?", backup_version, current_version\)', 'format!("Backup DB version (v{}) is higher than current version (v{}). Please upgrade the app first.", backup_version, current_version)'

# Line 752: AppError::config(format!("无法删除旧数据库: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to delete old database: {}", e))'

# Line 757: AppError::config(format!("无法添加数据库到 zip: {}", e))
$content = $content -replace 'AppError::config\(format!\(".*?: \{\}", e\)\)', 'AppError::config(format!("Failed to add database to zip: {}", e))'

# Line 1480: err_msg.contains("...") || err_msg.contains("...")
$content = $content -replace 'err_msg\.contains\(".*?"\) \|\| err_msg\.contains\(".*?ZIP"\)', 'err_msg.contains("Failed to open") || err_msg.contains("Failed to open ZIP")'

[System.IO.File]::WriteAllText($file, $content, [System.Text.UTF8Encoding]::new($false))
Write-Host "Done"
