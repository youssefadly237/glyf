# glyf

Look up Unicode glyphs by name (fuzzy search) or codepoint. Blazing fast.

## Usage

```bash
glyf <query>                # auto: hex/codepoint -> lookup, else fuzzy search
glyf -c <codepoint>         # exact codepoint lookup (279f, U+279F, 0x279F)
glyf -s <query>             # force name search
glyf -p <codepoint>         # record a pick in frecency db
glyf -l                     # list all entries
glyf -b <block>             # filter by Unicode block name
glyf -C <cat>               # filter by general category (Lu, Nd, L*, P*)
glyf -S <source>            # filter by source (unicode, nerdfonts)
glyf -I <set>               # filter by Nerd Fonts icon set (dev, fa, cod)
glyf --list-blocks          # list all Unicode blocks
glyf --list-categories      # list general categories
glyf --list-sources         # list all sources
glyf --list-icon-sets       # list icon sets with descriptions
glyf -t <n>                 # typo tolerance (default: 2, 0 = exact)
glyf -f <format>            # output format: plain, pretty, tsv
glyf -n <n>                 # max results (default: 50)
glyf --sort <field>         # sort by: relevance (default), name, codepoint
```

### Frecency

Frequently picked codepoints float to the top of empty and fuzzy searches.
Records are persisted to `$XDG_DATA_HOME/glyf/frecency.bin`

### Lua module

Build with `cargo build --features mlua --release`:

```lua
local glyf = require("glyf")

-- search: fuzzy name search with filters
glyf.search("lua", { limit = 10, sort = "name" })
glyf.search("", { block = "Arrows", category = "So" })
glyf.search("heart", { source = "nerdfonts", icon_set = "fa" })

-- lookup by codepoint string: U+0041, 0x279F, A, etc.
glyf.lookup("U+1F600")
glyf.lookup_name(codepoint)       -- name for a codepoint
glyf.category_of(codepoint)       -- general category
glyf.block_of(codepoint)          -- Unicode block name

-- list metadata
glyf.blocks()                     -- all Unicode blocks
glyf.categories()                 -- all general categories
glyf.sources()                    -- all sources (unicode, nerdfonts)
glyf.icon_sets()                  -- all icon sets with descriptions

-- frecency
glyf.record(0x1F600)              -- record a pick
glyf.frecency_get(0x0041)         -- get frecency count
glyf.frecency_path()              -- path to frecency db file
```

### Neovim plugin

[glyf.nvim](https://github.com/youssefadly237/glyf.nvim) - floating-window picker with live search, `@tag` filtering, and frecency.

## Planned

- Emoji (emoji-data.txt + CLDR annotations keywords)
- Reverse lookup for multi-character strings

## Data

- Unicode character database from [unicode.org](https://unicode.org/Public/UNIDATA/)
- Blocks: [Blocks.txt](https://unicode.org/Public/UNIDATA/Blocks.txt)
- Emoji: [emoji-data.txt](https://unicode.org/Public/UNIDATA/emoji/emoji-data.txt),
  [CLDR annotations](https://github.com/unicode-org/cldr-json/tree/main/cldr-json/cldr-annotations-full)
- Icons by [Material Design](https://github.com/google/material-design-icons)
