use std::io::{self, Write};
use std::str::FromStr;

use crate::corpus::{
    FIELD_ALT_NAME, FIELD_BIDI, FIELD_BLOCK, FIELD_CATEGORY, FIELD_COMBINING, FIELD_DECIMAL,
    FIELD_DECOMP, FIELD_DIGIT, FIELD_GLYPH, FIELD_ICON_SET, FIELD_LOWERCASE, FIELD_MIRRORED,
    FIELD_NUMERIC, FIELD_SOURCE, FIELD_TITLECASE, FIELD_UPPERCASE, Idx, codepoint, entry_name,
    entry_str,
};

pub enum Format {
    Plain,
    Pretty,
    Tsv,
}

impl FromStr for Format {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "pretty" => Ok(Self::Pretty),
            "tsv" => Ok(Self::Tsv),
            _ => Err("unknown format; use plain, pretty, or tsv"),
        }
    }
}

pub fn print_entry(w: &mut impl Write, idx: Idx, fmt: &Format) -> io::Result<()> {
    match fmt {
        Format::Plain => writeln!(w, "{}", entry_str(idx, FIELD_GLYPH)),
        Format::Pretty => writeln!(
            w,
            "{}\tU+{:04X}\t{}\t{}",
            entry_str(idx, FIELD_GLYPH),
            codepoint(idx),
            entry_name(idx),
            entry_str(idx, FIELD_BLOCK),
        ),
        Format::Tsv => writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            codepoint(idx),
            entry_str(idx, FIELD_GLYPH),
            entry_name(idx),
            entry_str(idx, FIELD_SOURCE),
            entry_str(idx, FIELD_CATEGORY),
            entry_str(idx, FIELD_COMBINING),
            entry_str(idx, FIELD_BIDI),
            entry_str(idx, FIELD_DECOMP),
            entry_str(idx, FIELD_DECIMAL),
            entry_str(idx, FIELD_DIGIT),
            entry_str(idx, FIELD_NUMERIC),
            entry_str(idx, FIELD_MIRRORED),
            entry_str(idx, FIELD_ALT_NAME),
            entry_str(idx, FIELD_UPPERCASE),
            entry_str(idx, FIELD_LOWERCASE),
            entry_str(idx, FIELD_TITLECASE),
            entry_str(idx, FIELD_BLOCK),
            entry_str(idx, FIELD_ICON_SET),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::lookup;

    fn sample_idx() -> Idx {
        lookup(0x0041).expect("A should be in corpus")
    }

    #[test]
    fn format_from_str_plain() {
        let f: Format = "plain".parse().expect("plain is valid");
        assert!(matches!(f, Format::Plain));
    }

    #[test]
    fn format_from_str_pretty() {
        let f: Format = "pretty".parse().expect("pretty is valid");
        assert!(matches!(f, Format::Pretty));
    }

    #[test]
    fn format_from_str_tsv() {
        let f: Format = "tsv".parse().expect("tsv is valid");
        assert!(matches!(f, Format::Tsv));
    }

    #[test]
    fn format_from_str_invalid() {
        assert!("badger".parse::<Format>().is_err());
    }

    #[test]
    fn print_plain_format() {
        let idx = sample_idx();
        let mut buf = Vec::new();
        print_entry(&mut buf, idx, &Format::Plain).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert_eq!(s, "A\n");
    }

    #[test]
    fn print_pretty_format() {
        let idx = sample_idx();
        let mut buf = Vec::new();
        print_entry(&mut buf, idx, &Format::Pretty).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert_eq!(s, "A\tU+0041\tLATIN CAPITAL LETTER A\tBasic Latin\n");
    }

    #[test]
    fn print_tsv_format() {
        let idx = sample_idx();
        let mut buf = Vec::new();
        print_entry(&mut buf, idx, &Format::Tsv).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert!(s.starts_with("65\tA\tLATIN CAPITAL LETTER A\tunicode"));
        assert!(s.ends_with("\tBasic Latin\t\n"));
    }
}
