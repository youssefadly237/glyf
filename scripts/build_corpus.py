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


def build(no_nerd_fonts: bool, no_fetch: bool) -> None:
    ensure("UnicodeData.txt", no_fetch)
    if not no_nerd_fonts:
        ensure("glyphnames.json", no_fetch)

    entries: list[
        tuple[
            int,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
            str,
        ]
    ] = []

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

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f, delimiter="\t")
        for entry in entries:
            writer.writerow(entry)

    print(f"done, {len(entries)} entries -> {OUT}")


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
