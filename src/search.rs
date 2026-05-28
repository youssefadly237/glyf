use std::sync::LazyLock;

use frizbee::{Config, match_list};

use crate::Sort;
use crate::corpus::{Entry, entries};
use crate::frecency::Frecency;

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub entry: &'a Entry<'static>,
    pub score: u16,
    pub freq: u16,
}

static NAMES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| entries().iter().map(|e| e.name).collect());

const MAX_QUERY_LEN: usize = 3275;

pub fn search<'a>(
    query: &str,
    entries: &'a [Entry<'static>],
    frecency: &Frecency,
    limit: usize,
    max_typos: Option<u16>,
    sort: Sort,
) -> Vec<Match<'a>> {
    if query.is_empty() {
        let mut ranked: Vec<Match<'a>> = entries
            .iter()
            .filter_map(|e| {
                let freq = frecency.get(e.codepoint);
                (freq > 0).then_some(Match {
                    entry: e,
                    score: 0,
                    freq: u32::min(freq, u16::MAX as u32) as u16,
                })
            })
            .collect();
        sort_matches(&mut ranked, sort);
        ranked.truncate(limit);
        return ranked;
    }

    if query.len() > MAX_QUERY_LEN {
        return Vec::new();
    }

    let max_allowed = query.len().saturating_sub(1);
    let max_allowed = u16::try_from(max_allowed).unwrap_or(u16::MAX);
    let max_typos = max_typos.map(|t| t.min(max_allowed));

    let query_upper = query.to_uppercase();

    let config = Config {
        max_typos,
        ..Config::default()
    };
    let raw = match_list(&query_upper, &NAMES, &config);

    let mut matches: Vec<Match<'a>> = raw
        .into_iter()
        .map(|m| {
            let entry = &entries[m.index as usize];
            Match {
                entry,
                score: m.score,
                freq: u32::min(frecency.get(entry.codepoint), u16::MAX as u32) as u16,
            }
        })
        .collect();

    sort_matches(&mut matches, sort);
    matches.truncate(limit);
    matches
}

fn sort_matches(matches: &mut [Match], sort: Sort) {
    match sort {
        Sort::Relevance => {
            matches.sort_unstable_by(|a, b| b.freq.cmp(&a.freq).then_with(|| b.score.cmp(&a.score)))
        }
        Sort::Name => matches.sort_unstable_by(|a, b| a.entry.name.cmp(b.entry.name)),
        Sort::Codepoint => {
            matches.sort_unstable_by(|a, b| a.entry.codepoint.cmp(&b.entry.codepoint))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_exact_finds_result() {
        let frec = Frecency::empty_for_testing();
        let results = search(
            "LATIN CAPITAL LETTER A",
            entries(),
            &frec,
            10,
            Some(0),
            Sort::Relevance,
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.name, "LATIN CAPITAL LETTER A");
    }

    #[test]
    fn search_fuzzy_finds_result() {
        let frec = Frecency::empty_for_testing();
        let results = search("GRINNING", entries(), &frec, 10, Some(2), Sort::Relevance);
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.entry.name == "GRINNING FACE"));
    }

    #[test]
    fn search_empty_query_returns_empty_with_cold_frecency() {
        let frec = Frecency::empty_for_testing();
        let results = search("", entries(), &frec, 10, None, Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_query_returns_frecency_hits() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        let results = search("", entries(), &frec, 10, None, Sort::Relevance);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.codepoint, 0x0041);
    }

    #[test]
    fn search_empty_query_respects_limit() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        frec.record(0x0042);
        let results = search("", entries(), &frec, 1, None, Sort::Relevance);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_too_long_query_returns_empty() {
        let frec = Frecency::empty_for_testing();
        let long_query = "A".repeat(MAX_QUERY_LEN + 1);
        let results = search(&long_query, entries(), &frec, 10, None, Sort::Relevance);
        assert!(results.is_empty());
    }

    #[test]
    fn search_no_results_returns_empty() {
        let frec = Frecency::empty_for_testing();
        let results = search(
            "XYZZY_NONEXISTENT",
            entries(),
            &frec,
            10,
            Some(0),
            Sort::Relevance,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn search_sort_by_name() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        frec.record(0x0042);
        let results = search("", entries(), &frec, 10, None, Sort::Name);
        assert_eq!(results.len(), 2);
        assert!(results[0].entry.name <= results[1].entry.name);
    }

    #[test]
    fn search_sort_by_codepoint() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0042);
        frec.record(0x0041);
        let results = search("", entries(), &frec, 10, None, Sort::Codepoint);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entry.codepoint, 0x0041);
        assert_eq!(results[1].entry.codepoint, 0x0042);
    }

    #[test]
    fn search_freq_clamped_to_u16() {
        let mut frec = Frecency::empty_for_testing();
        frec.record(0x0041);
        let results = search(
            "LATIN CAPITAL LETTER A",
            entries(),
            &frec,
            10,
            Some(0),
            Sort::Relevance,
        );
        assert!(!results.is_empty());
        assert!(results[0].freq <= u16::MAX);
    }

    #[test]
    fn search_index_within_bounds() {
        let frec = Frecency::empty_for_testing();
        let results = search("A", entries(), &frec, 10, Some(1), Sort::Relevance);
        for m in &results {
            assert!((m.entry.codepoint as usize) > 0);
        }
    }
}
