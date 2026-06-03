pub mod block;
pub mod category;
pub mod corpus;
pub mod frecency;
pub mod output;
pub mod search;

#[cfg(feature = "mlua")]
pub mod lua;

pub use corpus::{
    FIELD_GLYPH, Idx, category_of, codepoint, entry_block, entry_category, entry_icon_set,
    entry_name, entry_source, entry_str, icon_set_description, list_icon_sets, list_sources,
    lookup, lookup_name, lookup_str, num_entries, parse_cp_str,
};
pub use frecency::Frecency;
pub use output::Format;
pub use search::{Match, search_all, search_in};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    Relevance,
    Name,
    Codepoint,
}
