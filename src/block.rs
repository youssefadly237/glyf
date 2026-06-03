use std::cmp::Ordering;
use std::ops::Range;

use crate::corpus;

include!(concat!(env!("OUT_DIR"), "/blocks_data.rs"));

#[derive(Clone, Debug)]
pub struct Block {
    pub range: Range<u32>,
    pub name: &'static str,
}

pub fn all() -> &'static [Block] {
    BLOCKS
}

pub fn block_of(cp: u32) -> Option<&'static str> {
    BLOCKS
        .binary_search_by(|b| {
            if cp < b.range.start {
                Ordering::Greater
            } else if cp >= b.range.end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
        .map(|i| BLOCKS[i].name)
}

pub fn by_name(name: &str) -> Option<&'static Block> {
    BLOCKS.iter().find(|b| b.name.eq_ignore_ascii_case(name))
}

pub fn entry_range(range: &Range<u32>) -> Range<usize> {
    let start = corpus::lower_bound(range.start);
    let n = corpus::num_entries();
    let mut end = start;
    while end < n && corpus::codepoint(corpus::Idx(end as u32)) < range.end {
        end += 1;
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_of_first_cp() {
        assert_eq!(block_of(0x0000), Some("Basic Latin"));
    }

    #[test]
    fn block_of_last_cp() {
        assert_eq!(block_of(0x007F), Some("Basic Latin"));
    }

    #[test]
    fn block_of_boundary_next() {
        assert_eq!(block_of(0x0080), Some("Latin-1 Supplement"));
    }

    #[test]
    fn block_of_mid_block() {
        assert_eq!(block_of(0x0041), Some("Basic Latin"));
        assert_eq!(block_of(0x0600), Some("Arabic"));
        assert_eq!(block_of(0x1F600), Some("Emoticons"));
    }

    #[test]
    fn block_of_miss_between_blocks() {
        assert_eq!(block_of(0x15000), None);
    }

    #[test]
    fn block_of_surrogate() {
        assert_eq!(block_of(0xD800), Some("High Surrogates"));
    }

    #[test]
    fn by_name_case_insensitive() {
        let b = by_name("basic latin").expect("should find");
        assert_eq!(b.name, "Basic Latin");
        let b = by_name("BASIC LATIN").expect("should find");
        assert_eq!(b.name, "Basic Latin");
        let b = by_name("EMOTICONS").expect("should find");
        assert_eq!(b.name, "Emoticons");
    }

    #[test]
    fn by_name_unknown() {
        assert!(by_name("nonexistent block").is_none());
    }

    #[test]
    fn all_not_empty() {
        assert!(!all().is_empty());
    }

    #[test]
    fn block_of_across_blocks() {
        assert_eq!(block_of(0x007F), Some("Basic Latin"));
        assert_eq!(block_of(0x0080), Some("Latin-1 Supplement"));
        assert_eq!(block_of(0x00FF), Some("Latin-1 Supplement"));
        assert_eq!(block_of(0x0100), Some("Latin Extended-A"));
    }
}
