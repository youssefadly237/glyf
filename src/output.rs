use std::io::{self, Write};
use std::str::FromStr;

use crate::corpus::Entry;

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

pub fn print_entry(w: &mut impl Write, e: &Entry<'_>, fmt: &Format) -> io::Result<()> {
    match fmt {
        Format::Plain => writeln!(w, "{}", e.glyph),
        Format::Pretty => writeln!(w, "{}\tU+{:04X}\t{}", e.glyph, e.codepoint, e.name),
        Format::Tsv => writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            e.codepoint,
            e.glyph,
            e.name,
            e.source,
            e.category,
            e.combining,
            e.bidi,
            e.decomp,
            e.decimal,
            e.digit,
            e.numeric,
            e.mirrored,
            e.alt_name,
            e.uppercase,
            e.lowercase,
            e.titlecase,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> Entry<'static> {
        Entry {
            codepoint: 0x0041,
            glyph: "A",
            name: "LATIN CAPITAL LETTER A",
            source: "unicode",
            category: "Lu",
            combining: "0",
            bidi: "L",
            decomp: "",
            decimal: "",
            digit: "",
            numeric: "",
            mirrored: "N",
            alt_name: "",
            uppercase: "A",
            lowercase: "a",
            titlecase: "A",
        }
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
        let e = sample_entry();
        let mut buf = Vec::new();
        print_entry(&mut buf, &e, &Format::Plain).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert_eq!(s, "A\n");
    }

    #[test]
    fn print_pretty_format() {
        let e = sample_entry();
        let mut buf = Vec::new();
        print_entry(&mut buf, &e, &Format::Pretty).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert_eq!(s, "A\tU+0041\tLATIN CAPITAL LETTER A\n");
    }

    #[test]
    fn print_tsv_format() {
        let e = sample_entry();
        let mut buf = Vec::new();
        print_entry(&mut buf, &e, &Format::Tsv).expect("write should succeed");
        let s = String::from_utf8(buf).expect("buf should be utf-8");
        assert!(s.starts_with("65\tA\tLATIN CAPITAL LETTER A\tunicode"));
        assert!(s.ends_with("\tA\n"));
    }
}
