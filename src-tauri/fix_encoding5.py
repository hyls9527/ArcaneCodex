import re

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Context-aware replacements for garbled strings
# Map: (context_keyword, replacement_english)
# Applied line by line to avoid breaking code structure

replacements = {
    # backup_database function
    'db_path_clone.exists()': ('"Database file not found"', '"Database file not found"'),
    'File::create(&output_path_clone)': ('"Failed to create zip file"', '"Failed to create zip file: {}"'),
    'start_file(&db_file_name': ('"Failed to add database to zip"', '"Failed to add database to zip: {}"'),
    'wal_path.exists()': ('"Failed to add WAL file to zip"', '"Failed to add WAL file to zip: {}"'),
    'shm_path.exists()': ('"Failed to add SHM file to zip"', '"Failed to add SHM file to zip: {}"'),
    'zip.finalize()': ('"Failed to finalize zip"', '"Failed to finalize zip: {}"'),
    
    # restore_database function
    'ZipArchive::new': ('"Failed to open zip file"', '"Failed to open zip file: {}"'),
    'archive.by_index': ('"Failed to read zip entry"', '"Failed to read zip entry: {}"'),
    'archive.extract': ('"Failed to extract zip"', '"Failed to extract zip: {}"'),
    'std::fs::copy': ('"Failed to restore database"', '"Failed to restore database: {}"'),
    
    # General
    'create_dir_all': ('"Failed to create directory"', '"Failed to create directory: {}"'),
    'temp_dir': ('"Failed to create temp directory"', '"Failed to create temp directory: {}"'),
}

def has_garbled(line):
    """Check if line contains non-ASCII characters that look like garbled text"""
    return bool(re.search(r'[^\x00-\x7f]', line))

def fix_line(line):
    """Fix garbled strings in a single line"""
    if not has_garbled(line):
        return line
    
    # Find all quoted strings with non-ASCII
    def replace_garbled(m):
        full = m.group(0)
        inner = m.group(1)
        
        # Try to fix encoding
        for enc in ['latin-1', 'cp1252', 'iso-8859-1']:
            try:
                fixed = inner.encode(enc).decode('utf-8')
                if any('\u4e00' <= c <= '\u9fff' for c in fixed):
                    return '"' + fixed + '"'
            except:
                continue
        
        # Try raw byte interpretation
        try:
            b = bytes([ord(c) for c in inner if ord(c) < 256])
            if len(b) == len(inner):
                fixed = b.decode('utf-8')
                if any('\u4e00' <= c <= '\u9fff' for c in fixed):
                    return '"' + fixed + '"'
        except:
            pass
        
        return '"Operation failed"'
    
    line = re.sub(r'"([^"]*[^\x00-\x7f][^"]*)"', replace_garbled, line)
    return line

fixed_lines = [fix_line(line) for line in lines]

with open(filepath, 'w', encoding='utf-8') as f:
    f.writelines(fixed_lines)

print("Done fixing encoding line by line")
