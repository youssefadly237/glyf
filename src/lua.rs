use mlua::prelude::*;

use crate::Sort;
use crate::corpus::{
    self, FIELD_GLYPH, codepoint, entry_category, entry_name, entry_source, entry_str,
};
use crate::frecency::Frecency;
use crate::search;

fn entry_to_table(lua: &Lua, m: &search::Match) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    t.set("codepoint", codepoint(m.idx))?;
    t.set("glyph", entry_str(m.idx, FIELD_GLYPH))?;
    t.set("name", entry_name(m.idx))?;
    t.set("source", entry_source(m.idx))?;
    t.set("category", entry_category(m.idx))?;
    t.set("score", m.score)?;
    t.set("freq", m.freq)?;
    Ok(t)
}

fn do_search(lua: &Lua, (query, opts): (String, LuaTable)) -> LuaResult<LuaTable> {
    let limit: usize = opts.get("limit").unwrap_or(50);
    let max_typos: Option<u16> = opts.get("max_typos").unwrap_or_default();
    let sort_str: String = opts.get("sort").unwrap_or("relevance".to_string());
    let sort = match sort_str.as_str() {
        "relevance" | "score" => Sort::Relevance,
        "name" => Sort::Name,
        "codepoint" => Sort::Codepoint,
        _ => Sort::Relevance,
    };

    let frecency = Frecency::load();
    let results = search::search_all(&query, &frecency, limit, max_typos, sort);
    let out = lua.create_table()?;
    for (i, m) in results.iter().enumerate() {
        out.set(i + 1, entry_to_table(lua, m)?)?;
    }
    Ok(out)
}

fn lookup(lua: &Lua, query: String) -> LuaResult<Option<LuaTable>> {
    let frecency = Frecency::load();
    match corpus::lookup_str(&query) {
        Some(idx) => {
            let freq = u32::min(frecency.get(codepoint(idx)), u16::MAX as u32) as u16;
            let m = search::Match {
                idx,
                score: 0,
                freq,
            };
            Ok(Some(entry_to_table(lua, &m)?))
        }
        None => Ok(None),
    }
}

fn record(_lua: &Lua, codepoint: u32) -> LuaResult<()> {
    let mut frecency = Frecency::load();
    frecency.record(codepoint);
    frecency.flush().map_err(LuaError::external)
}

fn frecency_get(_lua: &Lua, codepoint: u32) -> LuaResult<u32> {
    let frecency = Frecency::load();
    Ok(frecency.get(codepoint))
}

fn frecency_path(_lua: &Lua, _: ()) -> LuaResult<String> {
    let path = Frecency::path();
    Ok(path.to_string_lossy().to_string())
}

#[mlua::lua_module]
fn glyf(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("search", lua.create_function(do_search)?)?;
    exports.set("lookup", lua.create_function(lookup)?)?;
    exports.set("record", lua.create_function(record)?)?;
    exports.set("frecency_get", lua.create_function(frecency_get)?)?;
    exports.set("frecency_path", lua.create_function(frecency_path)?)?;
    Ok(exports)
}
