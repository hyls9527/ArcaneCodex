import re

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'rb') as f:
    raw = f.read()

text = raw.decode('utf-8')

def try_all_fixes(s):
    """Try all possible encoding reversals"""
    # Common double-encoding chains
    chains = [
        ('latin-1', 'utf-8'),
        ('cp1252', 'utf-8'),
        ('iso-8859-1', 'utf-8'),
        ('mac_roman', 'utf-8'),
        ('cp1250', 'utf-8'),
        ('cp1251', 'utf-8'),
        ('cp850', 'utf-8'),
        ('cp437', 'utf-8'),
    ]
    
    for enc1, enc2 in chains:
        try:
            b = s.encode(enc1)
            fixed = b.decode(enc2)
            if any('\u4e00' <= c <= '\u9fff' for c in fixed):
                return fixed
        except:
            continue
    
    # Try: the garbled text might be raw UTF-8 bytes interpreted as something else
    # Encode the string as raw bytes (each char -> its Unicode code point as byte)
    # This only works if all chars are < 256
    try:
        b = bytes([ord(c) for c in s if ord(c) < 256])
        if len(b) == len(s):
            fixed = b.decode('utf-8')
            if any('\u4e00' <= c <= '\u9fff' for c in fixed):
                return fixed
    except:
        pass
    
    return None

def fix_match(m):
    inner = m.group(1)
    fixed = try_all_fixes(inner)
    if fixed:
        return '"' + fixed + '"'
    return '"Operation failed"'

pattern = r'"([^"]*[^\x00-\x7f][^"]*)"'
text = re.sub(pattern, fix_match, text)

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
