# Analyze garbled text to understand encoding chain
garbled = "闂佽桨鑳舵晶妤€鐣垫担瑙勫劅闁规儳鐡ㄩ悗顔济归悩铏瀯缂佹顦遍埀顒佺⊕閿氭繝鈧"

print("Garbled text:", repr(garbled))
print("Length:", len(garbled))
print()

# Show each character's code point
print("Code points:")
for i, c in enumerate(garbled):
    print(f"  [{i}] U+{ord(c):04X} ({c})")

print()

# Try to reverse: encode as various encodings, decode as UTF-8
encodings = ['latin-1', 'cp1252', 'iso-8859-1', 'mac_roman', 'cp1250', 'cp1251', 'cp850', 'cp437', 'cp1253', 'cp1254', 'cp1255', 'cp1256', 'cp1257', 'cp1258']

for enc in encodings:
    try:
        b = garbled.encode(enc)
        fixed = b.decode('utf-8')
        print(f"{enc}: {fixed}")
    except Exception as e:
        print(f"{enc}: ERROR - {e}")

print()

# Try: maybe the garbled text is the result of UTF-8 bytes being interpreted as GBK/GB2312
try:
    b = garbled.encode('gbk')
    fixed = b.decode('utf-8')
    print(f"gbk->utf-8: {fixed}")
except Exception as e:
    print(f"gbk->utf-8: ERROR - {e}")

# Try: maybe it's UTF-8 bytes interpreted as raw bytes then encoded as UTF-8
# Each char in garbled text -> get its UTF-8 bytes -> interpret as raw bytes -> decode as UTF-8
try:
    raw_bytes = garbled.encode('utf-8')
    print(f"Raw UTF-8 bytes: {raw_bytes.hex()}")
    # Try to interpret these as double-encoded
except Exception as e:
    print(f"Error: {e}")
