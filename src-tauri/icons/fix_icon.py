#!/usr/bin/env python3
"""Regenerate all icon sizes and ICO from the source PNG.
Ensures no black background leaks in and content is properly centered."""
import os, struct, io
from PIL import Image

DIR = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(DIR, "icon.png")

img = Image.open(SRC).convert("RGBA")

# Ensure white background (not transparent) for ICO
white_bg = Image.new("RGBA", img.size, (255, 255, 255, 255))
img = Image.alpha_composite(white_bg, img)

sizes = [16, 32, 48, 64, 128, 256, 512]

# Generate individual PNGs
for sz in sizes:
    resized = img.resize((sz, sz), Image.LANCZOS)
    out = os.path.join(DIR, f"{sz}x{sz}.png")
    resized.save(out)
    print(f"  {sz}x{sz}.png done")

# Generate ICO with embedded PNG images
entries = []
png_chunks = []
offset = 6 + len(sizes) * 16  # header + dir entries

for sz in sizes:
    resized = img.resize((sz, sz), Image.LANCZOS)
    buf = io.BytesIO()
    resized.save(buf, format="PNG")
    png_data = buf.getvalue()
    png_chunks.append(png_data)
    entries.append((sz, sz, len(png_data), offset))
    offset += len(png_data)

ico_path = os.path.join(DIR, "icon.ico")
with open(ico_path, "wb") as f:
    f.write(struct.pack("<HHH", 0, 1, len(sizes)))  # header
    for sz, h, length, off in entries:
        f.write(struct.pack("<BBBBHHII",
            0 if sz >= 256 else sz,  # width (0 = 256)
            0 if h >= 256 else h,    # height
            0, 0,                     # palette, reserved
            1, 32,                    # planes, bpp
            length, off               # size, offset
        ))
    for chunk in png_chunks:
        f.write(chunk)

print(f"\nicon.ico generated ({len(sizes)} sizes: {sizes})")
print("Done!")
