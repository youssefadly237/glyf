# glyf

Look up Unicode glyphs by name (fuzzy search) or codepoint. Blazing fast.

## Usage

```bash
glyf <query>                # auto: hex/codepoint -> lookup, else fuzzy search
glyf -c <codepoint>         # exact codepoint lookup (279f, U+279F, 0x279F)
glyf -s <query>             # force name search
glyf -p <codepoint>         # record a pick in frecency db
glyf -l                     # list all entries
glyf -t <n>                 # typo tolerance (default: 2, 0 = exact)
glyf -f <format>            # output format: plain, pretty, tsv
glyf -n <n>                 # max results (default: 50)
glyf --sort <field>         # sort by: relevance (default), name, codepoint
```

### Frecency

Frequently picked codepoints float to the top of empty and fuzzy searches.
Records are persisted to `$XDG_DATA_HOME/glyf/frecency.bin`

### Neovim plugin

coming soon

## Planned

- Blocks (Unicode block display, search by block)
- Emoji (emoji-data.txt + CLDR annotations keywords)
- ratatui TUI
- Reverse lookup for multi-character strings

## Data

- Unicode character database from [unicode.org](https://unicode.org/Public/UNIDATA/)
- Blocks: [Blocks.txt](https://unicode.org/Public/UNIDATA/Blocks.txt)
- Emoji: [emoji-data.txt](https://unicode.org/Public/UNIDATA/emoji/emoji-data.txt),
  [CLDR annotations](https://github.com/unicode-org/cldr-json/tree/main/cldr-json/cldr-annotations-full)
- Icons by [Material Design](https://github.com/google/material-design-icons)
