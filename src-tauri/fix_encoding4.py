import re

filepath = r"e:\ArcaneCodex\src-tauri\src\commands\settings.rs"

with open(filepath, 'rb') as f:
    raw = f.read()

# Fix double-encoded UTF-8 at byte level.
# Pattern: 0xC2 0x80-0xBF or 0xC3 0x80-0xBF
# These represent single bytes 0x00-0xBF or 0xC0-0xFF that were double-encoded.
# We need to find runs of these and decode them back.

result = bytearray()
i = 0
while i < len(raw):
    if i + 1 < len(raw) and raw[i] in (0xC2, 0xC3) and 0x80 <= raw[i+1] <= 0xBF:
        # Start of a potential double-encoded sequence
        # Collect all consecutive double-encoded bytes
        decoded_bytes = bytearray()
        j = i
        while j + 1 < len(raw) and raw[j] in (0xC2, 0xC3) and 0x80 <= raw[j+1] <= 0xBF:
            if raw[j] == 0xC2:
                decoded_bytes.append(raw[j+1])
            else:  # 0xC3
                decoded_bytes.append(raw[j+1] + 64)  # 0xC0-0xFF range
            j += 2
        
        # Try to decode as UTF-8
        try:
            decoded = decoded_bytes.decode('utf-8')
            # Check if it looks like valid Chinese text
            if any('\u4e00' <= c <= '\u9fff' for c in decoded):
                # Valid Chinese - keep the decoded version
                result.extend(decoded.encode('utf-8'))
                i = j
                continue
        except:
            pass
        
        # Not valid Chinese - keep original bytes
        result.extend(raw[i:j])
        i = j
    else:
        result.append(raw[i])
        i += 1

with open(filepath, 'wb') as f:
    f.write(bytes(result))

print("Done fixing encoding at byte level")
