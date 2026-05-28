use std::sync::LazyLock;

static CORPUS_TSV: &str = include_str!("../data/corpus.tsv");
static ENTRIES: LazyLock<Vec<Entry<'static>>> = LazyLock::new(|| parse_corpus(CORPUS_TSV));

pub fn parse_corpus(tsv: &str) -> Vec<Entry<'_>> {
    let mut entries: Vec<Entry<'_>> = tsv.lines().filter_map(Entry::parse).collect();
    entries.sort_unstable_by_key(|e| e.codepoint);
    entries
}

#[derive(Debug, Clone)]
pub struct Entry<'a> {
    pub codepoint: u32,
    pub glyph: &'a str,
    pub name: &'a str,
    pub source: &'a str,
    pub category: &'a str,
    pub combining: &'a str,
    pub bidi: &'a str,
    pub decomp: &'a str,
    pub decimal: &'a str,
    pub digit: &'a str,
    pub numeric: &'a str,
    pub mirrored: &'a str,
    pub alt_name: &'a str,
    pub uppercase: &'a str,
    pub lowercase: &'a str,
    pub titlecase: &'a str,
}

impl<'a> Entry<'a> {
    fn parse(line: &'a str) -> Option<Self> {
        let f: Vec<&'a str> = line.split('\t').collect();
        if f.len() < 4 {
            return None;
        }
        let codepoint = f[0].parse::<u32>().ok()?;
        Some(Entry {
            codepoint,
            glyph: f[1],
            name: f[2],
            source: f[3],
            category: f.get(4).unwrap_or(&""),
            combining: f.get(5).unwrap_or(&""),
            bidi: f.get(6).unwrap_or(&""),
            decomp: f.get(7).unwrap_or(&""),
            decimal: f.get(8).unwrap_or(&""),
            digit: f.get(9).unwrap_or(&""),
            numeric: f.get(10).unwrap_or(&""),
            mirrored: f.get(11).unwrap_or(&""),
            alt_name: f.get(12).unwrap_or(&""),
            uppercase: f.get(13).unwrap_or(&""),
            lowercase: f.get(14).unwrap_or(&""),
            titlecase: f.get(15).unwrap_or(&""),
        })
    }
}

pub fn entries() -> &'static [Entry<'static>] {
    &ENTRIES
}

pub fn lookup(cp: u32) -> Option<&'static Entry<'static>> {
    ENTRIES
        .binary_search_by_key(&cp, |e| e.codepoint)
        .ok()
        .map(|i| &ENTRIES[i])
}

pub fn lookup_str(s: &str) -> Option<&'static Entry<'static>> {
    let cp = parse_cp_str(s)?;
    lookup(cp)
}

pub fn parse_cp_str(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("U+").or_else(|| s.strip_prefix("u+")) {
        return u32::from_str_radix(rest, 16).ok();
    }
    if let Some(rest) = s.strip_prefix("0x") {
        return u32::from_str_radix(rest, 16).ok();
    }
    let first = s.chars().next()?;
    if s.len() == first.len_utf8() || !first.is_ascii() {
        return Some(u32::from(first));
    }
    u32::from_str_radix(s, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TSV: &str = "\
65\tA\tLATIN CAPITAL LETTER A\tunicode\tLu\t0\tL\t\t\t\t\tN\t\tA\tA\tA
66\tB\tLATIN CAPITAL LETTER B\tunicode\tLu\t0\tL\t\t\t\t\tN\t\tB\tB\tB
128513\t\u{1F600}\tGRINNING FACE\tunicode\tSo\t0\tON\t\t\t\t\tN\t\t\t\t
";

    #[test]
    fn parse_corpus_sorted_by_codepoint() {
        let entries = parse_corpus(SAMPLE_TSV);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].codepoint, 65);
        assert_eq!(entries[1].codepoint, 66);
        assert_eq!(entries[2].codepoint, 128513);
    }

    #[test]
    fn parse_corpus_short_line_skipped() {
        let entries = parse_corpus("1\tA\tfoo");
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_corpus_non_numeric_codepoint_skipped() {
        let entries = parse_corpus("xyz\tA\tfoo\tbar");
        assert!(entries.is_empty());
    }

    #[test]
    fn lookup_hit_lower_bound() {
        let e = lookup(0x0041).expect("A should exist");
        assert_eq!(e.name, "LATIN CAPITAL LETTER A");
    }

    #[test]
    fn lookup_hit_emoji() {
        let e = lookup(0x1F600).expect("grinning face should exist");
        assert_eq!(e.name, "GRINNING FACE");
    }

    #[test]
    fn lookup_miss_surrogate() {
        assert!(lookup(0xD800).is_none());
    }

    #[test]
    fn lookup_miss_above_range() {
        assert!(lookup(0xFFFFFF).is_none());
    }

    #[test]
    fn lookup_miss_unsassigned() {
        assert!(lookup(0x0378).is_none());
    }

    #[test]
    fn lookup_str_uplus_format() {
        let e = lookup_str("U+0041").expect("U+0041 should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_uplus_lowercase() {
        let e = lookup_str("u+0041").expect("u+0041 should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_0x_format() {
        let e = lookup_str("0x0041").expect("0x0041 should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_hex_only() {
        let e = lookup_str("0041").expect("0041 should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_single_ascii_char() {
        let e = lookup_str("A").expect("A should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_single_non_ascii_char() {
        let e = lookup_str("😀").expect("emoji should resolve");
        assert_eq!(e.codepoint, 0x1F600);
    }

    #[test]
    fn lookup_str_trimmed() {
        let e = lookup_str("  U+0041  ").expect("trimmed should resolve");
        assert_eq!(e.codepoint, 0x41);
    }

    #[test]
    fn lookup_str_not_found() {
        assert!(lookup_str("ZZZZ_NOT_A_CODEPOINT").is_none());
    }

    #[test]
    fn parse_cp_str_bare_hex_multi_char_ascii() {
        assert_eq!(parse_cp_str("0041"), Some(0x0041));
    }

    #[test]
    fn parse_cp_str_too_long_multi_char_ascii() {
        assert_eq!(parse_cp_str("AB"), Some(0xAB));
    }

    #[test]
    fn parse_cp_str_empty() {
        assert_eq!(parse_cp_str(""), None);
    }

    #[test]
    fn entries_returns_sorted() {
        let e = entries();
        for w in e.windows(2) {
            assert!(w[0].codepoint <= w[1].codepoint, "entries not sorted");
        }
    }

    #[test]
    fn entries_are_static() {
        let e1: &[Entry<'static>] = entries();
        let e2: &[Entry<'static>] = entries();
        assert!(std::ptr::eq(e1, e2));
    }
}
