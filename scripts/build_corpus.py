"""
Build a unified glyph corpus from Unicode and Nerd Fonts metadata.
"""

import json
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent
RAW = ROOT / "data" / "raw"
OUT = ROOT / "data" / "corpus.tsv"

RAW.mkdir(parents=True, exist_ok=True)

SOURCES = {
    "UnicodeData.txt": "https://www.unicode.org/Public/UCD/latest/ucd/UnicodeData.txt",
    "glyphnames.json": "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/master/glyphnames.json",
}

for filename, url in SOURCES.items():
    dest = RAW / filename
    print(f"  fetch {filename} ...")
    urllib.request.urlretrieve(url, dest)

entries = []

with open(RAW / "UnicodeData.txt", encoding="utf-8") as f:
    for line in f:
        fields = line.strip().split(";")
        if len(fields) < 15:
            continue
        name = fields[1]
        alt = fields[10].strip()
        if name.startswith("<"):
            if not alt:
                continue
            name = alt
        try:
            codepoint = int(fields[0], 16)
        except ValueError:
            continue
        if 0xD800 <= codepoint <= 0xDFFF:
            continue
        if 0xE000 <= codepoint <= 0xF8FF:
            continue
        entries.append(
            (
                codepoint,
                chr(codepoint),
                name,
                "unicode",
                fields[2],  # category
                fields[3],  # combining class
                fields[4],  # bidi class
                fields[5],  # decomposition
                fields[6],  # decimal value
                fields[7],  # digit value
                fields[8],  # numeric value
                fields[9],  # mirrored
                alt,  # unicode 1.0 name
                fields[12],  # uppercase mapping
                fields[13],  # lowercase mapping
                fields[14],  # titlecase mapping
            )
        )

with open(RAW / "glyphnames.json", encoding="utf-8") as f:
    data = json.load(f)

for key, val in data.items():
    if key == "METADATA":
        continue
    try:
        codepoint = int(val["code"], 16)
        glyph = val["char"]
    except (KeyError, ValueError):
        continue
    if not glyph:
        continue
    name = key.replace("-", " ").replace("_", " ").upper()
    # nerdfonts fields pad empty for unicode-only columns
    entries.append(
        (
            codepoint,
            glyph,
            name,
            "nerdfonts",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        )
    )

entries.sort(key=lambda e: e[0])

with open(OUT, "w", encoding="utf-8") as f:
    for entry in entries:
        f.write("\t".join(str(f) for f in entry) + "\n")

print(f"done, {len(entries)} entries -> {OUT}")
