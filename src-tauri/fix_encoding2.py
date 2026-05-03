import re

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

def fix_garbled(match):
    """Fix double-encoded UTF-8: replace garbled quote content only"""
    quote = match.group(1)
    
    try:
        fixed = quote.encode('latin-1').decode('utf-8')
        if any('\u4e00' <= c <= '\u9fff' or '\u3000' <= c <= '\u303f' for c in fixed):
            return '"' + fixed + '"'
    except:
        pass
    
    return '"Operation failed"'

# Match only the content inside quotes that contains non-ASCII
pattern = r'"([^"]*[^\x00-\x7f][^"]*)"'

fixed_content = re.sub(pattern, fix_garbled, content)

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(fixed_content)

print("Done fixing encoding")
