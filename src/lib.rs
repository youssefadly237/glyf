pub mod corpus;
pub mod frecency;
pub mod output;
pub mod search;

#[cfg(feature = "mlua")]
pub mod lua;

pub use corpus::{Entry, entries, lookup, lookup_str, parse_corpus, parse_cp_str};
pub use frecency::Frecency;
pub use output::Format;
pub use search::{Match, search};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    Relevance,
    Name,
    Codepoint,
}
