use std::io;
use std::io::Write;

use clap::{CommandFactory, Parser};
use glyf::{
    FIELD_GLYPH, Frecency, Idx, Sort, block, category, codepoint, entry_block, entry_category,
    entry_icon_set, entry_name, entry_source, entry_str, icon_set_description, list_icon_sets,
    list_sources, lookup_str, num_entries, output::print_entry, parse_cp_str, search_all,
    search_in,
};

#[derive(Parser)]
#[command(
    name = "glyf",
    about = "Look up Unicode glyphs by name or codepoint",
    version,
    after_help = "If the bare argument is a hex codepoint (279F, U+279F, 0x279F), looks it up.\n\
                  Otherwise, searches by name. Use -s to force name search."
)]
struct Args {
    #[arg(short = 's', long = "search", conflicts_with_all = &["codepoint", "pick", "list"], help = "Search by name (fuzzy)")]
    search: Option<String>,

    #[arg(short = 'c', long = "codepoint", conflicts_with_all = &["search", "pick", "list"], help = "Lookup by codepoint: 279f, U+279F, 0x279F")]
    codepoint: Option<String>,

    #[arg(short = 'p', long = "pick", conflicts_with_all = &["search", "codepoint", "list"], help = "Record a pick in frecency db")]
    pick: Option<String>,

    #[arg(short = 'l', long = "list", conflicts_with_all = &["search", "codepoint", "pick"], help = "List all entries")]
    list: bool,

    #[arg(short = 'b', long = "block", help = "Search within a Unicode block")]
    block: Option<String>,

    #[arg(long = "list-blocks", help = "List all Unicode blocks")]
    list_blocks: bool,

    #[arg(
        short = 'C',
        long = "category",
        help = "Filter by Unicode General Category (e.g. Lu, Nd, L* for all letters, P* for punctuation)"
    )]
    category: Vec<String>,

    #[arg(long = "list-categories", help = "List all Unicode General Categories")]
    list_categories: bool,

    #[arg(
        short = 'S',
        long = "source",
        help = "Filter by source (e.g. unicode, nerdfonts)"
    )]
    source: Vec<String>,

    #[arg(long = "list-sources", help = "List all sources")]
    list_sources: bool,

    #[arg(
        short = 'I',
        long = "icon-set",
        help = "Filter by Nerd Fonts icon set (e.g. dev, fa, cod)"
    )]
    icon_set: Vec<String>,

    #[arg(long = "list-icon-sets", help = "List all icon sets")]
    list_icon_sets: bool,

    #[arg(
        short = 'f',
        long = "format",
        default_value = "pretty",
        help = "Output format: pretty (default), plain, tsv"
    )]
    format: String,

    #[arg(
        short = 'n',
        long = "limit",
        default_value_t = 50,
        help = "Max results (default: 50)"
    )]
    limit: usize,

    #[arg(
        short = 't',
        long = "typos",
        default_value_t = 2,
        help = "Typo tolerance (default: 2, 0 = exact)"
    )]
    typos: u16,

    #[arg(
        long = "sort",
        default_value = "relevance",
        help = "Sort by: relevance (default, alias: score), name, codepoint"
    )]
    sort: String,

    #[arg(help = "Query: hex codepoint or name to search")]
    query: Option<String>,
}

fn print_help() -> ! {
    let mut out = io::stdout();
    let _ = Args::command().write_help(&mut out);
    let _ = writeln!(&mut out);
    std::process::exit(0);
}

fn exit_ok<T>(r: Result<T, impl std::fmt::Display>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

struct Filter {
    ranges: Vec<std::ops::Range<u32>>,
    sources: Vec<&'static str>,
    icon_sets: Vec<&'static str>,
    categories: Vec<&'static str>,
}

fn entry_in_bounds(idx: Idx, f: &Filter) -> bool {
    let cp = codepoint(idx);
    (f.ranges.is_empty() || f.ranges.iter().any(|r| r.contains(&cp)))
        && (f.sources.is_empty() || f.sources.contains(&entry_source(idx)))
        && (f.icon_sets.is_empty() || f.icon_sets.contains(&entry_icon_set(idx)))
        && (f.categories.is_empty() || f.categories.contains(&entry_category(idx)))
}

fn no_matches(query: &str) -> ! {
    eprintln!("no matches for {:?}", query);
    std::process::exit(1);
}

fn print_entries_pretty(out: &mut impl io::Write, indices: &[Idx]) {
    let names: Vec<&str> = indices.iter().map(|&idx| entry_name(idx)).collect();
    let max_name = names.iter().map(|n| n.len()).max().unwrap_or(0);
    for (&idx, &name) in indices.iter().zip(names.iter()) {
        let _ = writeln!(
            out,
            "{}\tU+{:04X}\t{:max_name$}\t{}",
            entry_str(idx, FIELD_GLYPH),
            codepoint(idx),
            name,
            entry_block(idx),
        );
    }
}

fn print_results(out: &mut impl io::Write, results: &[glyf::Match], fmt: &glyf::Format) -> usize {
    if matches!(fmt, glyf::Format::Pretty) {
        let indices: Vec<Idx> = results.iter().map(|m| m.idx).collect();
        print_entries_pretty(out, &indices);
        return results.len();
    }
    for m in results {
        let _ = print_entry(out, m.idx, fmt);
    }
    results.len()
}

fn do_search(query: &str, limit: usize, typos: u16, sort: Sort, fmt: &glyf::Format, f: &Filter) {
    let frecency = Frecency::load();
    if f.ranges.is_empty()
        && f.sources.is_empty()
        && f.icon_sets.is_empty()
        && f.categories.is_empty()
    {
        let results = search_all(query, &frecency, limit, Some(typos), sort);
        if results.is_empty() {
            no_matches(query);
        }
        print_results(&mut io::stdout().lock(), &results, fmt);
        return;
    }
    let base: Box<dyn Iterator<Item = Idx>> = if f.ranges.is_empty() {
        Box::new((0..num_entries()).map(|i| Idx(i as u32)))
    } else {
        Box::new(
            f.ranges
                .iter()
                .flat_map(|r| block::entry_range(r).map(|i| Idx(i as u32))),
        )
    };
    let pool: Vec<Idx> = base.filter(|&idx| entry_in_bounds(idx, f)).collect();
    if pool.is_empty() {
        no_matches(query);
    }

    let names: Vec<&str> = pool.iter().map(|&idx| entry_name(idx)).collect();
    let results = search_in(query, &pool, &names, &frecency, limit, Some(typos), sort);
    if results.is_empty() {
        no_matches(query);
    }
    print_results(&mut io::stdout().lock(), &results, fmt);
}

fn validate_str_list<'a>(
    known: &[&'a str],
    input: &[String],
    vec: &mut Vec<&'a str>,
    label: &str,
) -> Result<(), String> {
    for s in input {
        match known.iter().find(|k| *k == s) {
            Some(&k) => {
                if !vec.contains(&k) {
                    vec.push(k);
                }
            }
            None => return Err(format!("unknown {label} '{s}'")),
        }
    }
    Ok(())
}

fn resolve_filter(args: &Args) -> Result<Filter, String> {
    let mut f = Filter {
        ranges: vec![],
        sources: vec![],
        icon_sets: vec![],
        categories: vec![],
    };

    if let Some(ref name) = args.block {
        f.ranges.push(
            block::by_name(name)
                .map(|b| b.range.clone())
                .ok_or_else(|| format!("unknown block '{}'", name))?,
        );
    }

    for pat in &args.category {
        f.categories.extend(category::resolve(pat)?);
    }

    validate_str_list(list_sources(), &args.source, &mut f.sources, "source")?;
    validate_str_list(
        list_icon_sets(),
        &args.icon_set,
        &mut f.icon_sets,
        "icon set",
    )?;

    Ok(f)
}

fn main() {
    let args = Args::parse();

    let fmt = exit_ok(args.format.parse::<glyf::Format>());
    let sort = match args.sort.as_str() {
        "relevance" | "score" => Sort::Relevance,
        "name" => Sort::Name,
        "codepoint" => Sort::Codepoint,
        _ => {
            eprintln!(
                "error: unknown sort '{}'; use relevance, name, or codepoint",
                args.sort
            );
            std::process::exit(1);
        }
    };

    if args.list_blocks {
        let blks = block::all();
        let pad = blks.iter().map(|b| b.name.len()).max().unwrap_or(0);
        for b in blks {
            println!(
                "{name:<pad$}  U+{start:04X}..U+{end:04X}",
                name = b.name,
                pad = pad,
                start = b.range.start,
                end = b.range.end - 1,
            );
        }
        return;
    }
    if args.list_categories {
        for c in category::list() {
            println!("{}\t{}", c.code, c.desc);
        }
        return;
    }
    if args.list_sources {
        for s in list_sources() {
            println!("{}", s);
        }
        return;
    }
    if args.list_icon_sets {
        for s in list_icon_sets() {
            let desc = icon_set_description(s);
            println!("{s:12}  {desc}");
        }
        return;
    }

    let filter = exit_ok(resolve_filter(&args));

    if let Some(cp_str) = args.codepoint {
        let entry = lookup_str(&cp_str);
        match entry {
            Some(idx) if entry_in_bounds(idx, &filter) => {
                drop(print_entry(&mut io::stdout(), idx, &fmt))
            }
            _ => {
                eprintln!("not found: {cp_str}");
                std::process::exit(1);
            }
        }
    } else if let Some(query) = args.search {
        do_search(&query, args.limit, args.typos, sort, &fmt, &filter);
    } else if let Some(cp_str) = args.pick {
        let Some(cp) = parse_cp_str(&cp_str) else {
            eprintln!("bad codepoint: {cp_str}");
            std::process::exit(1);
        };
        let mut frecency = Frecency::load();
        frecency.record(cp);
        if let Err(e) = frecency.flush() {
            eprintln!("error: failed to save frecency: {e}");
            std::process::exit(1);
        }
    } else if args.list {
        let mut sorted: Vec<Idx> = (0..num_entries())
            .map(|i| Idx(i as u32))
            .filter(|&idx| entry_in_bounds(idx, &filter))
            .collect();
        match sort {
            Sort::Name => sorted.sort_unstable_by(|&a, &b| entry_name(a).cmp(entry_name(b))),
            Sort::Codepoint => sorted.sort_unstable_by(|&a, &b| codepoint(a).cmp(&codepoint(b))),
            Sort::Relevance => {}
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        if matches!(fmt, glyf::Format::Pretty) {
            print_entries_pretty(&mut out, &sorted);
        } else {
            for &idx in &sorted {
                let _ = print_entry(&mut out, idx, &fmt);
            }
        }
    } else if let Some(q) = args.query {
        let entry = lookup_str(&q);
        if let Some(idx) = entry
            && entry_in_bounds(idx, &filter)
        {
            drop(print_entry(&mut io::stdout(), idx, &fmt));
        } else {
            do_search(&q, args.limit, args.typos, sort, &fmt, &filter);
        }
    } else {
        print_help();
    }
}
