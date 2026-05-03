import re
import sys

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'rb') as f:
    raw = f.read()

# Try to fix double-encoded UTF-8:
# The garbled text happens when UTF-8 bytes are interpreted as Latin-1,
# then re-encoded as UTF-8.
# To reverse: decode as Latin-1, then encode back to get original bytes.
try:
    # Decode the raw bytes as UTF-8 first
    text = raw.decode('utf-8')
    
    # Find all string literals that contain garbled Chinese
    # Garbled Chinese chars are in the range of common double-encoding patterns
    def fix_double_encoded(match):
        s = match.group(1)
        try:
            # Try to reverse double encoding: encode as latin-1, decode as utf-8
            fixed = s.encode('latin-1').decode('utf-8')
            return '"' + fixed + '"'
        except:
            return match.group(0)
    
    # Match string literals (both "..." and format!("...", ...))
    # This is a simplified approach - match quoted strings
    pattern = r'"([^"]*[\x80-\xff][^"]*)"'
    text = re.sub(pattern, fix_double_encoded, text)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(text)
    
    print("Fixed encoding")
except Exception as e:
    print(f"Error: {e}")
