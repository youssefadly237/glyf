use std::io;
use std::io::Write;

use clap::{CommandFactory, Parser};
use glyf::{Frecency, Sort, entries, lookup_str, output::print_entry, parse_cp_str, search};

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

fn run_search(query: &str, limit: usize, typos: u16, sort: Sort, fmt: &glyf::Format) {
    let frecency = Frecency::load();
    let results = search(query, entries(), &frecency, limit, Some(typos), sort);
    if results.is_empty() {
        eprintln!("no matches for {:?}", query);
        std::process::exit(1);
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for m in &results {
        let _ = print_entry(&mut out, m.entry, fmt);
    }
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

    if let Some(cp_str) = args.codepoint {
        match lookup_str(&cp_str) {
            Some(entry) => drop(print_entry(&mut io::stdout(), entry, &fmt)),
            None => {
                eprintln!("not found: {cp_str}");
                std::process::exit(1);
            }
        }
    } else if let Some(query) = args.search {
        run_search(&query, args.limit, args.typos, sort, &fmt);
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
        let mut entries: Vec<&glyf::Entry<'_>> = entries().iter().collect();
        match sort {
            Sort::Name => entries.sort_unstable_by(|a, b| a.name.cmp(b.name)),
            Sort::Codepoint => entries.sort_unstable_by(|a, b| a.codepoint.cmp(&b.codepoint)),
            Sort::Relevance => {}
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for entry in entries {
            let _ = print_entry(&mut out, entry, &fmt);
        }
    } else if let Some(q) = args.query {
        match lookup_str(&q) {
            Some(entry) => drop(print_entry(&mut io::stdout(), entry, &fmt)),
            None => run_search(&q, args.limit, args.typos, sort, &fmt),
        }
    } else {
        print_help();
    }
}
