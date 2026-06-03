use std::cmp::Ordering;

include!(concat!(env!("OUT_DIR"), "/field_consts.rs"));
include!(concat!(env!("OUT_DIR"), "/binary_data.rs"));
include!(concat!(env!("OUT_DIR"), "/metadata_data.rs"));
include!(concat!(env!("OUT_DIR"), "/category_data.rs"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Idx(pub u32);

fn get_u32(data: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(data[i * 4..][..4].try_into().unwrap())
}

fn get_u16(data: &[u8], i: usize) -> u16 {
    u16::from_le_bytes(data[i * 2..][..2].try_into().unwrap())
}

pub fn num_entries() -> usize {
    CODEPOINT_DATA.len() / 4
}

pub fn codepoint(idx: Idx) -> u32 {
    get_u32(CODEPOINT_DATA, idx.0 as usize)
}

pub fn entry_str(idx: Idx, field: usize) -> &'static str {
    let off = get_u32(OFFSET_DATA, idx.0 as usize * NUM_FIELDS + field) as usize;
    let len = get_u16(LENGTH_DATA, idx.0 as usize * NUM_FIELDS + field) as usize;
    std::str::from_utf8(&STRING_DATA[off..off + len]).unwrap()
}

pub fn entry_name(idx: Idx) -> &'static str {
    let off = get_u32(NAME_OFFSET_DATA, idx.0 as usize);
    let len = get_u16(NAME_LENGTH_DATA, idx.0 as usize);
    std::str::from_utf8(&STRING_DATA[off as usize..][..len as usize]).unwrap()
}
pub fn entry_source(idx: Idx) -> &'static str {
    entry_str(idx, FIELD_SOURCE)
}
pub fn entry_category(idx: Idx) -> &'static str {
    entry_str(idx, FIELD_CATEGORY)
}
pub fn entry_block(idx: Idx) -> &'static str {
    entry_str(idx, FIELD_BLOCK)
}
pub fn entry_icon_set(idx: Idx) -> &'static str {
    entry_str(idx, FIELD_ICON_SET)
}

pub fn category_of(cp: u32) -> Option<&'static str> {
    CATEGORY_DATA
        .binary_search_by(|&(start, end, _)| {
            if cp < start {
                Ordering::Greater
            } else if cp > end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
        .map(|i| CATEGORY_DATA[i].2)
}

pub(crate) fn lower_bound(cp: u32) -> usize {
    let data = CODEPOINT_DATA;
    let n = data.len() / 4;
    let mut left = 0usize;
    let mut right = n;
    while left < right {
        let mid = left + (right - left) / 2;
        if get_u32(data, mid) < cp {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

pub fn lookup(cp: u32) -> Option<Idx> {
    let i = lower_bound(cp);
    let data = CODEPOINT_DATA;
    if i < data.len() / 4 && get_u32(data, i) == cp {
        Some(Idx(i as u32))
    } else {
        None
    }
}

include!(concat!(env!("OUT_DIR"), "/name_lookup.rs"));

pub fn lookup_name(cp: u32) -> Option<&'static str> {
    NAME_LOOKUP
        .binary_search_by_key(&cp, |&(c, _)| c)
        .ok()
        .map(|i| NAME_LOOKUP[i].1)
}

pub fn lookup_str(s: &str) -> Option<Idx> {
    let cp = parse_cp_str(s)?;
    lookup(cp)
}

pub fn list_sources() -> &'static [&'static str] {
    SOURCES
}

pub fn list_icon_sets() -> &'static [&'static str] {
    ICON_SETS
}

pub fn icon_set_description(name: &str) -> &'static str {
    match name {
        "cod" => "Codicons",
        "custom" => "Seti and original",
        "dev" => "Devicons",
        "extra" => "Extra glyphs",
        "fa" => "Font Awesome",
        "fae" => "Font Awesome Extension",
        "iec" => "Power Symbols IEC",
        "indent" | "indentation" => "Extra glyphs",
        "linux" => "Font Logos",
        "md" => "Material Design",
        "oct" => "Octicons",
        "pl" => "Powerline Symbols",
        "ple" => "Powerline Extra",
        "pom" => "Pomicons",
        "seti" => "Seti and original",
        "weather" => "Weather Icons",
        _ => "",
    }
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

    #[test]
    fn lookup_hit_lower_bound() {
        let idx = lookup(0x0041).expect("A should exist");
        assert_eq!(entry_name(idx), "LATIN CAPITAL LETTER A");
    }

    #[test]
    fn lookup_hit_emoji() {
        let idx = lookup(0x1F600).expect("grinning face should exist");
        assert_eq!(entry_name(idx), "GRINNING FACE");
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
        let idx = lookup_str("U+0041").expect("U+0041 should resolve");
        assert_eq!(codepoint(idx), 0x41);
    }

    #[test]
    fn lookup_str_uplus_lowercase() {
        let idx = lookup_str("u+0041").expect("u+0041 should resolve");
        assert_eq!(codepoint(idx), 0x41);
    }

    #[test]
    fn lookup_str_0x_format() {
        let idx = lookup_str("0x0041").expect("0x0041 should resolve");
        assert_eq!(codepoint(idx), 0x41);
    }

    #[test]
    fn lookup_str_hex_only() {
        let idx = lookup_str("0041").expect("0041 should resolve");
        assert_eq!(codepoint(idx), 0x41);
    }

    #[test]
    fn lookup_str_single_ascii_char() {
        let idx = lookup_str("A").expect("A should resolve");
        assert_eq!(codepoint(idx), 0x41);
    }

    #[test]
    fn lookup_str_single_non_ascii_char() {
        let idx = lookup_str("\u{1F600}").expect("emoji should resolve");
        assert_eq!(codepoint(idx), 0x1F600);
    }

    #[test]
    fn lookup_str_trimmed() {
        let idx = lookup_str("  U+0041  ").expect("trimmed should resolve");
        assert_eq!(codepoint(idx), 0x41);
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
    fn entries_are_sorted() {
        let n = num_entries();
        for i in 1..n {
            let prev = codepoint(Idx(i as u32 - 1));
            let cur = codepoint(Idx(i as u32));
            assert!(
                prev <= cur,
                "entries not sorted at index {i}: {prev} > {cur}"
            );
        }
    }

    #[test]
    fn entries_lookup_roundtrip() {
        let idx = lookup(0x0041).expect("A should exist");
        assert_eq!(codepoint(idx), 0x0041);
        assert_eq!(entry_str(idx, FIELD_GLYPH), "A");
        assert_eq!(entry_name(idx), "LATIN CAPITAL LETTER A");
    }
}
