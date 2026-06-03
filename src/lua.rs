use mlua::prelude::*;

use crate::Sort;
use crate::corpus::{self, entries};
use crate::frecency::Frecency;
use crate::search;

fn entry_to_table(lua: &Lua, m: &search::Match<'_>) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    t.set("codepoint", m.entry.codepoint)?;
    t.set("glyph", m.entry.glyph)?;
    t.set("name", m.entry.name)?;
    t.set("source", m.entry.source)?;
    t.set("category", m.entry.category)?;
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
    let results = search::search(&query, entries(), &frecency, limit, max_typos, sort);
    let out = lua.create_table()?;
    for (i, m) in results.iter().enumerate() {
        out.set(i + 1, entry_to_table(lua, m)?)?;
    }
    Ok(out)
}

fn lookup(lua: &Lua, query: String) -> LuaResult<Option<LuaTable>> {
    let frecency = Frecency::load();
    match corpus::lookup_str(&query) {
        Some(entry) => {
            let freq = u32::min(frecency.get(entry.codepoint), u16::MAX as u32) as u16;
            let m = search::Match {
                entry,
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
