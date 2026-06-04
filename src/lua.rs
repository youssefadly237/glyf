use mlua::prelude::*;

use std::ops::Range;

use crate::Sort;
use crate::block;
use crate::category;
use crate::corpus::{
    self, FIELD_GLYPH, Idx, category_of, codepoint, entry_category, entry_icon_set, entry_name,
    entry_source, entry_str, icon_set_description, list_icon_sets, list_sources, lookup_name,
    num_entries,
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
    t.set("block", corpus::entry_block(m.idx))?;
    t.set("icon_set", corpus::entry_icon_set(m.idx))?;
    t.set("score", m.score)?;
    t.set("freq", m.freq)?;
    Ok(t)
}

fn split_csv(s: &str) -> Vec<&str> {
    s.split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect()
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

    let block_names: Option<String> = opts.get("block").ok();
    let cat_pats: Option<String> = opts.get("category").ok();
    let raw_range: Option<(u32, u32)> = opts
        .get::<LuaTable>("range")
        .ok()
        .and_then(|t| Some((t.get(1).ok()?, t.get(2).ok()?)));
    let sources: Option<String> = opts.get("source").ok();
    let icon_sets: Option<String> = opts.get("icon_set").ok();

    let has_filter = block_names.is_some()
        || cat_pats.is_some()
        || raw_range.is_some()
        || sources.is_some()
        || icon_sets.is_some();

    let frecency = Frecency::load();

    let results = if has_filter {
        let mut ranges: Vec<Range<u32>> = Vec::new();
        if let Some(ref names) = block_names {
            for name in split_csv(names) {
                let b = block::by_name(name.trim())
                    .ok_or_else(|| LuaError::external(format!("unknown block '{name}'")))?;
                ranges.push(b.range.clone());
            }
        }
        if let Some((start, end)) = raw_range {
            ranges.push(start..end + 1);
        }

        let mut cats: Vec<&str> = Vec::new();
        if let Some(ref pats) = cat_pats {
            for pat in split_csv(pats) {
                let resolved = category::resolve(pat.trim()).map_err(LuaError::external)?;
                cats.extend(resolved);
            }
        }

        let source_list: Vec<&str> = sources.as_ref().map(|s| split_csv(s)).unwrap_or_default();
        let icon_set_list: Vec<&str> = icon_sets.as_ref().map(|s| split_csv(s)).unwrap_or_default();

        let base: Box<dyn Iterator<Item = Idx>> = if ranges.is_empty() {
            Box::new((0..num_entries()).map(|i| Idx(i as u32)))
        } else {
            Box::new(
                ranges
                    .iter()
                    .flat_map(|r| block::entry_range(r).map(|i| Idx(i as u32))),
            )
        };
        let pool: Vec<Idx> = base
            .filter(|&idx| {
                (cats.is_empty() || cats.contains(&entry_category(idx)))
                    && (source_list.is_empty() || source_list.contains(&entry_source(idx)))
                    && (icon_set_list.is_empty() || icon_set_list.contains(&entry_icon_set(idx)))
            })
            .collect();

        if pool.is_empty() {
            return lua.create_table();
        }
        let names: Vec<&str> = pool.iter().map(|&idx| entry_name(idx)).collect();
        search::search_in(&query, &pool, &names, &frecency, limit, max_typos, sort)
    } else {
        search::search_all(&query, &frecency, limit, max_typos, sort)
    };

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

fn blocks(_lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let t = _lua.create_table()?;
    for (i, b) in block::all().iter().enumerate() {
        let btab = _lua.create_table()?;
        btab.set("name", b.name)?;
        btab.set("start", b.range.start)?;
        btab.set("end", b.range.end - 1)?;
        t.set(i + 1, btab)?;
    }
    Ok(t)
}

fn block_of(_lua: &Lua, cp: u32) -> LuaResult<Option<&'static str>> {
    Ok(block::block_of(cp))
}

fn categories(_lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let t = _lua.create_table()?;
    for (i, c) in category::list().iter().enumerate() {
        let ctab = _lua.create_table()?;
        ctab.set("code", c.code)?;
        ctab.set("desc", c.desc)?;
        t.set(i + 1, ctab)?;
    }
    Ok(t)
}

fn sources(_lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let t = _lua.create_table()?;
    for (i, s) in list_sources().iter().enumerate() {
        t.set(i + 1, *s)?;
    }
    Ok(t)
}

fn icon_sets(_lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let t = _lua.create_table()?;
    for (i, s) in list_icon_sets().iter().enumerate() {
        let stab = _lua.create_table()?;
        let code: &str = s;
        stab.set("code", code)?;
        stab.set("desc", icon_set_description(s))?;
        t.set(i + 1, stab)?;
    }
    Ok(t)
}

fn name_lookup(_lua: &Lua, cp: u32) -> LuaResult<Option<&'static str>> {
    Ok(lookup_name(cp))
}

fn category_lookup(_lua: &Lua, cp: u32) -> LuaResult<Option<&'static str>> {
    Ok(category_of(cp))
}

#[mlua::lua_module]
fn glyf_core(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("search", lua.create_function(do_search)?)?;
    exports.set("lookup", lua.create_function(lookup)?)?;
    exports.set("record", lua.create_function(record)?)?;
    exports.set("frecency_get", lua.create_function(frecency_get)?)?;
    exports.set("frecency_path", lua.create_function(frecency_path)?)?;
    exports.set("blocks", lua.create_function(blocks)?)?;
    exports.set("block_of", lua.create_function(block_of)?)?;
    exports.set("categories", lua.create_function(categories)?)?;
    exports.set("sources", lua.create_function(sources)?)?;
    exports.set("icon_sets", lua.create_function(icon_sets)?)?;
    exports.set("lookup_name", lua.create_function(name_lookup)?)?;
    exports.set("category_of", lua.create_function(category_lookup)?)?;
    Ok(exports)
}
