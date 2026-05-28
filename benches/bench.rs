use std::sync::LazyLock;

static FREC: LazyLock<glyf::Frecency> = LazyLock::new(glyf::Frecency::load);
static POPULATED_FREC: LazyLock<glyf::Frecency> = LazyLock::new(|| {
    let mut f = glyf::Frecency::load();
    for cp in [0x0041, 0x0042, 0x0043, 0x1F600, 0x00E9] {
        f.record(cp);
    }
    f
});

fn main() {
    glyf::entries();
    LazyLock::force(&FREC);
    divan::main();
}

#[divan::bench]
fn lookup_hit() -> Option<&'static glyf::Entry<'static>> {
    glyf::lookup(divan::black_box(0x0041))
}

#[divan::bench]
fn lookup_miss() -> Option<&'static glyf::Entry<'static>> {
    glyf::lookup(divan::black_box(0xFFFF))
}

#[divan::bench]
fn lookup_emoji() -> Option<&'static glyf::Entry<'static>> {
    glyf::lookup(divan::black_box(0x1F600))
}

#[divan::bench]
fn entries_ref() -> &'static [glyf::Entry<'static>] {
    glyf::entries()
}

#[divan::bench]
fn parse_full_corpus() -> Vec<glyf::Entry<'static>> {
    let tsv = include_str!("../data/corpus.tsv");
    glyf::parse_corpus(divan::black_box(tsv))
}

#[divan::bench]
fn search_fuzzy() -> Vec<glyf::Match<'static>> {
    glyf::search(
        divan::black_box("musical"),
        glyf::entries(),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(2)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn search_short() -> Vec<glyf::Match<'static>> {
    glyf::search(
        divan::black_box("a"),
        glyf::entries(),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(1)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn search_exact() -> Vec<glyf::Match<'static>> {
    glyf::search(
        divan::black_box("MUSICAL SYMBOL COMBINING DOIT"),
        glyf::entries(),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(0)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn search_empty() -> Vec<glyf::Match<'static>> {
    glyf::search(
        divan::black_box(""),
        glyf::entries(),
        &POPULATED_FREC,
        divan::black_box(50),
        divan::black_box(None),
        glyf::Sort::Relevance,
    )
}
