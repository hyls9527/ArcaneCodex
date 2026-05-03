import re

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

replacements = [
    (r'format!\("Operation failed", e\)', r'format!("Operation failed: {}", e)'),
    (r'format!\("Failed to process WAL file", e\)', r'format!("Failed to process WAL file: {}", e)'),
    (r'format!\("Failed to process SHM file", e\)', r'format!("Failed to process SHM file: {}", e)'),
    (r'format!\("Database operation failed",\s*db_file_name\s*\)', r'format!("Database operation failed: {}", db_file_name)'),
]

for pattern, replacement in replacements:
    count = len(re.findall(pattern, content))
    content = re.sub(pattern, replacement, content)
    print(f"Fixed {count} occurrences")

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

print("Done!")
