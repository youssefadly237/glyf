"""
Build a unified glyph corpus from Unicode and Nerd Fonts metadata.
"""

import argparse
import csv
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent
RAW = ROOT / "data" / "raw"
OUT = ROOT / "data" / "corpus.tsv"

UNICODE_VERSION = "18.0.0"
NERD_FONTS_REF = "v3.4.0"

SOURCES = {
    "UnicodeData.txt": {
        "url": f"https://www.unicode.org/Public/{UNICODE_VERSION}/ucd/UnicodeData.txt",
    },
    "glyphnames.json": {
        "url": f"https://raw.githubusercontent.com/ryanoasis/nerd-fonts/{NERD_FONTS_REF}/glyphnames.json",
    },
    "Blocks.txt": {
        "url": f"https://www.unicode.org/Public/{UNICODE_VERSION}/ucd/Blocks.txt",
    },
}


def fetch(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    for attempt in range(3):
        try:
            print(f"  fetch {dest.name} (attempt {attempt + 1}) ...")
            urllib.request.urlretrieve(url, dest)
            return
        except (urllib.error.URLError, urllib.error.HTTPError, OSError) as e:
            print(f"  error: {e}", file=sys.stderr)
            if attempt < 2:
                wait = 2**attempt
                print(f"  retrying in {wait}s ...")
                time.sleep(wait)
    print("  failed after 3 attempts", file=sys.stderr)
    sys.exit(1)


def ensure(name: str, no_fetch: bool) -> Path:
    dest = RAW / name
    if not dest.exists():
        if no_fetch:
            print(f"  {name} not found (--no-fetch set)", file=sys.stderr)
            sys.exit(1)
        fetch(SOURCES[name]["url"], dest)
    return dest


def parse_blocks() -> list[tuple[int, int, str]]:
    """Parse Blocks.txt into sorted list of (start, end, name)."""
    blocks: list[tuple[int, int, str]] = []
    with open(RAW / "Blocks.txt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or line.startswith("@"):
                continue
            parts = line.split(";")
            if len(parts) < 2:
                continue
            range_part, name = parts[0].strip(), parts[1].strip()
            start_hex, _, end_hex = range_part.partition("..")
            try:
                start = int(start_hex, 16)
                end = int(end_hex, 16)
            except ValueError:
                continue
            blocks.append((start, end, name))
    return blocks


def block_for(cp: int, blocks: list[tuple[int, int, str]]) -> str:
    lo, hi = 0, len(blocks)
    while lo < hi:
        mid = (lo + hi) // 2
        s, e, _ = blocks[mid]
        if cp < s:
            hi = mid
        elif cp > e:
            lo = mid + 1
        else:
            return blocks[mid][2]
    return ""


# Block names whose entries come from <First, Last> range expansion
EXPANDED_BLOCKS = set(
    line.strip()
    for line in (ROOT / "data" / "expanded_blocks.txt").read_text(encoding="utf-8").splitlines()
    if line.strip()
)


def build(no_nerd_fonts: bool, no_fetch: bool) -> None:
    ensure("UnicodeData.txt", no_fetch)
    ensure("Blocks.txt", no_fetch)
    if not no_nerd_fonts:
        ensure("glyphnames.json", no_fetch)

    blocks = parse_blocks()

    entries: list[tuple] = []
    range_saved: list = []  # [start, name_prefix, fields_2..14]

    with open(RAW / "UnicodeData.txt", encoding="utf-8") as f:
        for line in f:
            fields = line.strip().split(";")
            if len(fields) < 15:
                continue
            name = fields[1]
            alt = fields[10].strip()
            try:
                codepoint = int(fields[0], 16)
            except ValueError:
                continue

            # <First, Last> range markers
            if name.startswith("<") and "First" in name:
                if 0xD800 <= codepoint <= 0xDFFF or 0xE000 <= codepoint <= 0xF8FF or codepoint >= 0xF0000:
                    range_saved.clear()
                    continue
                range_name = name.split(",")[0].lstrip("<").strip()
                range_saved = [
                    codepoint, range_name,
                    fields[2], fields[3], fields[4], fields[5],
                    fields[6], fields[7], fields[8], fields[9],
                    fields[10], fields[12], fields[13], fields[14],
                ]
                continue

            if name.startswith("<") and "Last" in name:
                if range_saved:
                    start = range_saved[0]
                    r_name = range_saved[1]
                    for cp in range(start, codepoint + 1):
                        block = block_for(cp, blocks)
                        c_name = f"{r_name.upper()}-{cp:04X}"
                        entries.append((
                            cp, "", c_name, "unicode",
                            range_saved[2], range_saved[3], range_saved[4], range_saved[5],
                            range_saved[6], range_saved[7], range_saved[8], range_saved[9],
                            range_saved[10], range_saved[11], range_saved[12], range_saved[13],
                            block, "",
                        ))
                range_saved.clear()
                continue

            # Normal single entry
            if name.startswith("<"):
                if not alt:
                    continue
                name = alt
            if 0xD800 <= codepoint <= 0xDFFF:
                continue
            if 0xE000 <= codepoint <= 0xF8FF:
                continue
            block = block_for(codepoint, blocks)
            entries.append(
                (
                    codepoint,
                    "",
                    name,
                    "unicode",
                    fields[2],
                    fields[3],
                    fields[4],
                    fields[5],
                    fields[6],
                    fields[7],
                    fields[8],
                    fields[9],
                    alt,
                    fields[12],
                    fields[13],
                    fields[14],
                    block,
                    "",
                )
            )
    if not no_nerd_fonts:
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
                icon_set = key.split("-")[0]
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
                        "PUA",
                        icon_set,
                    )
                )

    entries.sort(key=lambda e: e[0])

    OUT.parent.mkdir(parents=True, exist_ok=True)
    HEADER = [
        "codepoint", "glyph", "name", "source", "category",
        "combining", "bidi", "decomp", "decimal", "digit",
        "numeric", "mirrored", "alt_name", "uppercase",
        "lowercase", "titlecase", "block", "icon_set",
    ]
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f, delimiter="\t")
        writer.writerow(HEADER)
        for entry in entries:
            writer.writerow(entry)

    blocks_tsv = ROOT / "data" / "blocks.tsv"
    with open(blocks_tsv, "w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f, delimiter="\t")
        for start, end, name in blocks:
            writer.writerow((f"{start:04X}", f"{end:04X}", name))

    print(f"done, {len(entries)} entries -> {OUT}")
    print(f"done, {len(blocks)} blocks -> {blocks_tsv}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build glyph corpus from Unicode and Nerd Fonts data"
    )
    parser.add_argument(
        "--no-nerd-fonts", action="store_true", help="Skip Nerd Fonts glyphs"
    )
    parser.add_argument(
        "--no-fetch",
        action="store_true",
        help="Fail if source files are missing (offline mode)",
    )
    args = parser.parse_args()
    build(no_nerd_fonts=args.no_nerd_fonts, no_fetch=args.no_fetch)


if __name__ == "__main__":
    main()
