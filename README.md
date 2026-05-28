# glyf

Look up Unicode glyphs by name (fuzzy search) or codepoint. Blazing fast.

## Usage

```bash
glyf <query>         # bare, search by name, fall back to codepoint
glyf -c <codepoint>  # exact codepoint lookup
glyf -s <query>      # force name search
glyf -p <codepoint>  # pick by codepoint, print one field
glyf -l              # list all entries
glyf -f <format>     # output format: plain, pretty, tsv
glyf -n <n>          # max results
glyf <char>          # reverse lookup (character to info)
```

## Planned

- Blocks (Unicode block display, search by block)
- Emoji (emoji-data.txt + CLDR annotations keywords)
- CLI output format options
- ratatui TUI
- Reverse lookup for multi-character strings

## Data

- Unicode character database from [unicode.org](https://unicode.org/Public/UNIDATA/)
- Blocks: [Blocks.txt](https://unicode.org/Public/UNIDATA/Blocks.txt)
- Emoji: [emoji-data.txt](https://unicode.org/Public/UNIDATA/emoji/emoji-data.txt),
  [CLDR annotations](https://github.com/unicode-org/cldr-json/tree/main/cldr-json/cldr-annotations-full)
- Icons by [Material Design](https://github.com/google/material-design-icons)
