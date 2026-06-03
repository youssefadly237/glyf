use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

fn read_expanded_ranges() -> Vec<(u32, u32)> {
    let expanded: Vec<String> = std::fs::read_to_string("data/expanded_blocks.txt")
        .expect("data/expanded_blocks.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    let tsv = std::fs::read_to_string("data/blocks.tsv").expect("data/blocks.tsv");
    let mut ranges = Vec::new();
    for line in tsv.lines() {
        let mut parts = line.split('\t');
        let start_hex = parts.next().unwrap_or("");
        let end_hex = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("").trim();
        if expanded.iter().any(|e| e == name) {
            let start = u32::from_str_radix(start_hex, 16).unwrap_or(0);
            let end = u32::from_str_radix(end_hex, 16).unwrap_or(0);
            ranges.push((start, end));
        }
    }
    ranges
}

fn col_idx(header: &[&str], name: &str) -> usize {
    header.iter().position(|&c| c == name).expect(name)
}

fn write_corpus(out: &Path, expanded: &[(u32, u32)]) {
    let cjk_enabled = std::env::var("CARGO_FEATURE_CJK").is_ok();
    let tsv = std::fs::read_to_string("data/corpus.tsv").expect("data/corpus.tsv");
    let mut tsv_lines = tsv.lines();
    let header: Vec<&str> = tsv_lines.next().unwrap_or("").split('\t').collect();

    let i_cp = col_idx(&header, "codepoint");
    let i_glyph = col_idx(&header, "glyph");
    let i_name = col_idx(&header, "name");

    let mut lines: Vec<Vec<&str>> = Vec::new();
    for line in tsv_lines {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() == header.len() && f[i_cp].parse::<u32>().is_ok() {
            if !cjk_enabled {
                let cp: u32 = f[i_cp].parse().unwrap();
                if expanded.iter().any(|&(s, e)| cp >= s && cp <= e) {
                    continue;
                }
            }
            lines.push(f);
        }
    }

    let mut string_data = Vec::<u8>::new();
    let mut string_map: HashMap<String, (u32, u16)> = HashMap::new();
    for fields in &lines {
        let cp: u32 = fields[i_cp].parse().unwrap();
        let glyph = char::from_u32(cp)
            .map(|c| c.to_string())
            .unwrap_or_default();
        for (i, field) in fields.iter().enumerate().skip(1) {
            let val = if i == i_glyph { glyph.as_str() } else { field };
            if !string_map.contains_key(val) {
                let offset = string_data.len() as u32;
                string_data.extend_from_slice(val.as_bytes());
                string_map.insert(val.to_string(), (offset, val.len() as u16));
            }
        }
    }

    std::fs::write(out.join("string_data.bin"), &string_data).unwrap();

    let mut off_f = std::fs::File::create(out.join("field_offsets.bin")).unwrap();
    let mut len_f = std::fs::File::create(out.join("field_lengths.bin")).unwrap();
    let mut cp_f = std::fs::File::create(out.join("codepoints.bin")).unwrap();
    let mut noff_f = std::fs::File::create(out.join("name_offsets.bin")).unwrap();
    let mut nlen_f = std::fs::File::create(out.join("name_lengths.bin")).unwrap();
    for fields in &lines {
        let cp: u32 = fields[i_cp].parse().unwrap();
        cp_f.write_all(&cp.to_le_bytes()).unwrap();
        let glyph = char::from_u32(cp)
            .map(|c| c.to_string())
            .unwrap_or_default();
        for (i, field) in fields.iter().enumerate().skip(1) {
            let val = if i == i_glyph { glyph.as_str() } else { field };
            let (off, len) = string_map[val];
            off_f.write_all(&off.to_le_bytes()).unwrap();
            len_f.write_all(&len.to_le_bytes()).unwrap();
            if i == i_name {
                noff_f.write_all(&off.to_le_bytes()).unwrap();
                nlen_f.write_all(&len.to_le_bytes()).unwrap();
            }
        }
    }

    let num = lines.len();

    write_binary_data_rs(out);
    write_field_consts(out, &header);
    write_name_lookup(out, &header, &lines);
    write_metadata_rs(out, &header, &lines);
    write_category_data(out, &header, &lines);
    write_category_codes(out, &header, &lines);

    eprintln!(
        "wrote {} entries, {} bytes string data",
        num,
        string_data.len()
    );
}

fn write_metadata_rs(out: &Path, header: &[&str], lines: &[Vec<&str>]) {
    let i_src = col_idx(header, "source");
    let i_icon = col_idx(header, "icon_set");

    let mut sources: Vec<&str> = lines.iter().map(|f| f[i_src]).collect();
    sources.sort_unstable();
    sources.dedup();

    let mut icon_sets: Vec<&str> = lines
        .iter()
        .filter_map(|f| {
            if f[i_icon].is_empty() {
                None
            } else {
                Some(f[i_icon])
            }
        })
        .collect();
    icon_sets.sort_unstable();
    icon_sets.dedup();

    let mut s = String::new();
    s.push_str("pub const SOURCES: &[&str] = &[\n");
    for src in &sources {
        s.push_str(&format!("    {:?},\n", src));
    }
    s.push_str("];\n");
    s.push_str("pub const ICON_SETS: &[&str] = &[\n");
    for is in &icon_sets {
        s.push_str(&format!("    {:?},\n", is));
    }
    s.push_str("];\n");
    std::fs::write(out.join("metadata_data.rs"), s).unwrap();
    eprintln!(
        "wrote {} sources, {} icon sets",
        sources.len(),
        icon_sets.len()
    );
}

fn write_category_data(out: &Path, header: &[&str], lines: &[Vec<&str>]) {
    let i_cp = col_idx(header, "codepoint");
    let i_cat = col_idx(header, "category");

    let mut ranges: Vec<(u32, u32, &str)> = Vec::new();
    for fields in lines {
        let cp: u32 = fields[i_cp].parse().unwrap();
        let cat = fields[i_cat];
        if let Some(last) = ranges.last_mut()
            && last.2 == cat
            && cp == last.1 + 1
        {
            last.1 = cp;
            continue;
        }
        ranges.push((cp, cp, cat));
    }
    let mut s = String::from("pub const CATEGORY_DATA: &[(u32, u32, &str)] = &[\n");
    for &(start, end, cat) in &ranges {
        s.push_str(&format!("    (0x{:X}, 0x{:X}, {:?}),\n", start, end, cat));
    }
    s.push_str("];\n");
    std::fs::write(out.join("category_data.rs"), s).unwrap();
    eprintln!("wrote {} category ranges", ranges.len());
}

fn write_binary_data_rs(out: &Path) {
    let files = [
        ("string_data.bin", "STRING_DATA"),
        ("codepoints.bin", "CODEPOINT_DATA"),
        ("field_offsets.bin", "OFFSET_DATA"),
        ("field_lengths.bin", "LENGTH_DATA"),
        ("name_offsets.bin", "NAME_OFFSET_DATA"),
        ("name_lengths.bin", "NAME_LENGTH_DATA"),
    ];
    let mut s = String::new();
    for (fname, cname) in &files {
        s.push_str(&format!(
            "static {cname}: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{fname}\"));\n"
        ));
    }
    std::fs::write(out.join("binary_data.rs"), s).unwrap();
}

fn write_field_consts(out: &Path, header: &[&str]) {
    let mut s = String::new();
    for (i, col) in header.iter().enumerate().skip(1) {
        if *col == "name" {
            continue; // entry_name() provides direct access
        }
        let const_name = col.to_uppercase().replace('-', "_");
        s.push_str(&format!(
            "pub const FIELD_{const_name}: usize = {};\n",
            i - 1
        ));
    }
    s.push_str(&format!(
        "pub const NUM_FIELDS: usize = {};\n",
        header.len() - 1
    ));
    std::fs::write(out.join("field_consts.rs"), s).unwrap();
    eprintln!(
        "wrote {} field constants, NUM_FIELDS = {}",
        header.len() - 2,
        header.len() - 1
    );
}

fn write_category_codes(out: &Path, header: &[&str], lines: &[Vec<&str>]) {
    let i_cat = col_idx(header, "category");
    let mut codes: Vec<&str> = lines.iter().map(|f| f[i_cat]).collect();
    codes.sort_unstable();
    codes.dedup();
    let mut s = String::from("pub(crate) const CATS: &[Cat] = &[\n");
    for &code in &codes {
        let desc = match code {
            "Lu" => "Uppercase Letter",
            "Ll" => "Lowercase Letter",
            "Lt" => "Titlecase Letter",
            "Lm" => "Modifier Letter",
            "Lo" => "Other Letter",
            "Mn" => "Nonspacing Mark",
            "Mc" => "Spacing Combining Mark",
            "Me" => "Enclosing Mark",
            "Nd" => "Decimal Number",
            "Nl" => "Letter Number",
            "No" => "Other Number",
            "Pc" => "Connector Punctuation",
            "Pd" => "Dash Punctuation",
            "Ps" => "Open Punctuation",
            "Pe" => "Close Punctuation",
            "Pi" => "Initial Punctuation",
            "Pf" => "Final Punctuation",
            "Po" => "Other Punctuation",
            "Sm" => "Math Symbol",
            "Sc" => "Currency Symbol",
            "Sk" => "Modifier Symbol",
            "So" => "Other Symbol",
            "Zs" => "Space Separator",
            "Zl" => "Line Separator",
            "Zp" => "Paragraph Separator",
            "Cc" => "Control",
            "Cf" => "Format",
            "Cs" => "Surrogate",
            "Co" => "Private Use",
            "Cn" => "Unassigned",
            _ => "",
        };
        s.push_str(&format!(
            "    Cat {{ code: {:?}, desc: {:?} }},\n",
            code, desc
        ));
    }
    s.push_str("];\n");
    std::fs::write(out.join("category_codes.rs"), s).unwrap();
    eprintln!("wrote {} category codes", codes.len());
}

fn write_name_lookup(out: &Path, header: &[&str], lines: &[Vec<&str>]) {
    let i_cp = col_idx(header, "codepoint");
    let i_name = col_idx(header, "name");

    let mut s = String::from("pub(crate) static NAME_LOOKUP: &[(u32, &str)] = &[\n");
    for fields in lines {
        let cp: u32 = fields[i_cp].parse().unwrap();
        s.push_str(&format!("    (0x{:X}, {:?}),\n", cp, fields[i_name]));
    }
    s.push_str("];\n");
    std::fs::write(out.join("name_lookup.rs"), s).unwrap();
    eprintln!("wrote {} name lookups", lines.len());
}

fn write_blocks_rs(out: &Path) {
    let tsv = std::fs::read_to_string("data/blocks.tsv").expect("data/blocks.tsv");
    let mut s = String::from("pub const BLOCKS: &[Block] = &[\n");
    for line in tsv.lines() {
        let mut parts = line.split('\t');
        let start_hex = parts.next().unwrap_or("");
        let end_hex = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("").trim();
        let start = u32::from_str_radix(start_hex, 16).unwrap_or(0);
        let end = u32::from_str_radix(end_hex, 16).unwrap_or(0);
        s.push_str(&format!(
            "    Block {{ range: {}..{}, name: {:?} }},\n",
            start,
            end + 1,
            name
        ));
    }
    s.push_str("];\n");
    std::fs::write(out.join("blocks_data.rs"), s).unwrap();
    eprintln!("wrote {} blocks", tsv.lines().count());
}

fn main() {
    println!("cargo:rerun-if-changed=data/corpus.tsv");
    println!("cargo:rerun-if-changed=data/blocks.tsv");
    println!("cargo:rerun-if-changed=data/expanded_blocks.txt");
    println!("cargo:rerun-if-changed=scripts/build_corpus.py");

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let expanded = read_expanded_ranges();
    write_corpus(&out, &expanded);
    write_blocks_rs(&out);
}
