use std::sync::LazyLock;

use frizbee::{Config, match_list};

use crate::Sort;
use crate::corpus::{Idx, codepoint, entry_name, num_entries};
use crate::frecency::Frecency;

#[derive(Debug, Clone)]
pub struct Match {
    pub idx: Idx,
    pub score: u16,
    pub freq: u16,
}

static NAMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let n = num_entries();
    (0..n).map(|i| entry_name(Idx(i as u32))).collect()
});

// frizbee panics if needle longer than this (u16 score overflow).
const MAX_QUERY_LEN: usize = 3275;

fn search_empty(
    indices: impl IntoIterator<Item = Idx>,
    frecency: &Frecency,
    limit: usize,
    sort: Sort,
) -> Vec<Match> {
    let mut ranked: Vec<Match> = indices
        .into_iter()
        .filter_map(|idx| {
            let freq = frecency.get(codepoint(idx));
            (freq > 0).then_some(Match {
                idx,
                score: 0,
                freq: u32::min(freq, u16::MAX as u32) as u16,
            })
        })
        .collect();
    sort_matches(&mut ranked, sort);
    ranked.truncate(limit);
    ranked
}

fn search_impl(
    query: &str,
    names: &[&str],
    frecency: &Frecency,
    limit: usize,
    max_typos: Option<u16>,
    sort: Sort,
    get_idx: impl Fn(u32) -> Idx,
) -> Vec<Match> {
    let max_allowed = query.len().saturating_sub(1);
    let max_allowed = u16::try_from(max_allowed).unwrap_or(u16::MAX);
    let max_typos = max_typos.map(|t| t.min(max_allowed));
    let query_upper = query.to_uppercase();
    let config = Config {
        max_typos,
        ..Config::default()
    };
    let raw = match_list(&query_upper, names, &config);
    let mut matches: Vec<Match> = raw
        .into_iter()
        .map(|m| {
            let idx = get_idx(m.index);
            Match {
                idx,
                score: m.score,
                freq: u32::min(frecency.get(codepoint(idx)), u16::MAX as u32) as u16,
            }
        })
        .collect();
    sort_matches(&mut matches, sort);
    matches.truncate(limit);
    matches
}

pub fn search_in(
    query: &str,
    indices: &[Idx],
    names: &[&str],
    frecency: &Frecency,
    limit: usize,
    max_typos: Option<u16>,
    sort: Sort,
) -> Vec<Match> {
    debug_assert_eq!(
        indices.len(),
        names.len(),
        "indices and names must be aligned"
    );
    if query.is_empty() {
        return search_empty(indices.iter().copied(), frecency, limit, sort);
    }
    if query.len() > MAX_QUERY_LEN {
        return Vec::new();
    }
    search_impl(query, names, frecency, limit, max_typos, sort, |i| {
        indices[i as usize]
    })
}

pub fn search_all(
    query: &str,
    frecency: &Frecency,
    limit: usize,
    max_typos: Option<u16>,
    sort: Sort,
) -> Vec<Match> {
    if query.is_empty() {
        return search_empty(
            (0..num_entries()).map(|i| Idx(i as u32)),
            frecency,
            limit,
            sort,
        );
    }
    if query.len() > MAX_QUERY_LEN {
        return Vec::new();
    }
    search_impl(query, &NAMES, frecency, limit, max_typos, sort, Idx)
}

fn sort_matches(matches: &mut [Match], sort: Sort) {
    match sort {
        Sort::Relevance => {
            matches.sort_unstable_by(|a, b| b.freq.cmp(&a.freq).then_with(|| b.score.cmp(&a.score)))
        }
        Sort::Name => matches.sort_by_cached_key(|m| entry_name(m.idx)),
        Sort::Codepoint => matches.sort_unstable_by(|a, b| codepoint(a.idx).cmp(&codepoint(b.idx))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_exact_finds_result() {
        let frec = Frecency::empty_for_testing();
        let results = search_all(
            "LATIN CAPITAL LETTER A",
            &frec,
            10,
            Some(0),
            Sort::Relevance,
        );
        assert!(!results.is_empty());
        assert_eq!(entry_name(results[0].idx), "LATIN CAPITAL LETTER A");
    }

    #[test]
    fn search_fuzzy_finds_result() {
        let frec = Frecency::empty_for_testing();
        let results = search_all("GRINNING", &frec, 10, Some(2), Sort::Relevance);
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| entry_name(m.idx) == "GRINNING FACE"));
    }

    #[test]
    fn search_empty_query_returns_empty_with_cold_frecency() {
        let frec = Frecency::empty_for_testing();
        let results = search_all("", &frec, 10, None, Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_query_returns_frecency_hits() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        let results = search_all("", &frec, 10, None, Sort::Relevance);
        assert_eq!(results.len(), 1);
        assert_eq!(codepoint(results[0].idx), 0x0041);
    }

    #[test]
    fn search_empty_query_respects_limit() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        frec.record(0x0042);
        let results = search_all("", &frec, 1, None, Sort::Relevance);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_too_long_query_returns_empty() {
        let frec = Frecency::empty_for_testing();
        let long_query = "A".repeat(MAX_QUERY_LEN + 1);
        let results = search_all(&long_query, &frec, 10, None, Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_no_results_returns_empty() {
        let frec = Frecency::empty_for_testing();
        let results = search_all("XYZZY_NONEXISTENT", &frec, 10, Some(0), Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_sort_by_name() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        frec.record(0x0042);
        let results = search_all("", &frec, 10, None, Sort::Name);
        assert_eq!(results.len(), 2);
        assert!(entry_name(results[0].idx) <= entry_name(results[1].idx));
    }

    #[test]
    fn search_sort_by_codepoint() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0042);
        frec.record(0x0041);
        let results = search_all("", &frec, 10, None, Sort::Codepoint);
        assert_eq!(results.len(), 2);
        assert_eq!(codepoint(results[0].idx), 0x0041);
        assert_eq!(codepoint(results[1].idx), 0x0042);
    }

    #[test]
    fn search_freq_no_clamp_needed_for_u16() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        let results = search_all(
            "LATIN CAPITAL LETTER A",
            &frec,
            10,
            Some(0),
            Sort::Relevance,
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn search_in_restricted_pool() {
        let frec = Frecency::empty_for_testing();
        let idx = crate::corpus::lookup(0x0041).expect("A must be in corpus");
        let indices = vec![idx];
        let names = vec![crate::corpus::entry_name(idx)];
        let results = search_in("A", &indices, &names, &frec, 10, Some(0), Sort::Relevance);
        assert_eq!(results.len(), 1);
        assert_eq!(codepoint(results[0].idx), 0x0041);
    }

    #[test]
    fn search_in_empty_pool_yields_empty() {
        let frec = Frecency::empty_for_testing();
        let results = search_in("A", &[], &[], &frec, 10, Some(0), Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_index_within_bounds() {
        let frec = Frecency::empty_for_testing();
        let results = search_all("A", &frec, 10, Some(1), Sort::Relevance);
        for m in &results {
            assert!(codepoint(m.idx) > 0);
        }
    }
}
