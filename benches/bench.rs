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
    LazyLock::force(&FREC);
    divan::main();
}

#[divan::bench]
fn glyf_lookup_hit() -> Option<glyf::Idx> {
    glyf::lookup(divan::black_box(0x0041))
}

#[divan::bench]
fn glyf_lookup_miss() -> Option<glyf::Idx> {
    glyf::lookup(divan::black_box(0xFFFF))
}

#[divan::bench]
fn glyf_lookup_emoji() -> Option<glyf::Idx> {
    glyf::lookup(divan::black_box(0x1F600))
}

#[divan::bench]
fn glyf_search_fuzzy() -> Vec<glyf::Match> {
    glyf::search_all(
        divan::black_box("musical"),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(2)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn glyf_search_short() -> Vec<glyf::Match> {
    glyf::search_all(
        divan::black_box("a"),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(1)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn glyf_search_exact() -> Vec<glyf::Match> {
    glyf::search_all(
        divan::black_box("MUSICAL SYMBOL COMBINING DOIT"),
        &FREC,
        divan::black_box(10),
        divan::black_box(Some(0)),
        glyf::Sort::Relevance,
    )
}

#[divan::bench]
fn glyf_search_empty() -> Vec<glyf::Match> {
    glyf::search_all(
        divan::black_box(""),
        &POPULATED_FREC,
        divan::black_box(50),
        divan::black_box(None),
        glyf::Sort::Relevance,
    )
}

fn glyf_name_of(cp: u32) -> Option<&'static str> {
    glyf::lookup_name(cp)
}

#[divan::bench]
fn glyf_name_ascii() -> Option<&'static str> {
    glyf_name_of(divan::black_box(0x0041))
}

#[divan::bench]
fn glyf_name_emoji() -> Option<&'static str> {
    glyf_name_of(divan::black_box(0x1F600))
}

fn glyf_category_of(cp: u32) -> Option<&'static str> {
    glyf::category_of(cp)
}

#[divan::bench]
fn glyf_category_ascii() -> Option<&'static str> {
    glyf_category_of(divan::black_box(0x0041))
}

#[divan::bench]
fn glyf_category_emoji() -> Option<&'static str> {
    glyf_category_of(divan::black_box(0x1F600))
}

#[divan::bench]
fn glyf_block_ascii() -> Option<&'static str> {
    glyf::block::block_of(divan::black_box(0x0041))
}

#[divan::bench]
fn glyf_block_emoji() -> Option<&'static str> {
    glyf::block::block_of(divan::black_box(0x1F600))
}

#[divan::bench]
fn glyf_block_cjk() -> Option<&'static str> {
    glyf::block::block_of(divan::black_box(0x4E00))
}

#[divan::bench]
fn glyf_entry_name_ascii() -> &'static str {
    let idx = glyf::lookup(divan::black_box(0x0041)).unwrap();
    glyf::entry_name(idx)
}

#[divan::bench]
fn glyf_entry_name_emoji() -> &'static str {
    let idx = glyf::lookup(divan::black_box(0x1F600)).unwrap();
    glyf::entry_name(idx)
}

#[divan::bench]
fn glyf_block_of_boundary() -> Option<&'static str> {
    glyf::block::block_of(divan::black_box(0x007F))
}
