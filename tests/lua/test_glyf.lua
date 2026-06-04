local mod_dir = arg and arg[1] or '.'
package.cpath = mod_dir .. '/lib?.so'
local glyf = require('glyf')

local passed = 0
local failed = 0

local function test(name, fn)
  local ok, err = pcall(fn)
  if ok then
    io.write(string.format('test %-50s ... ok\n', name))
    passed = passed + 1
  else
    io.write(string.format('test %-50s ... FAILED\n', name))
    io.stderr:write('  ' .. tostring(err) .. '\n')
    failed = failed + 1
  end
end

test('search returns results', function()
  local r = glyf.search('A', { limit = 3 })
  assert(#r > 0 and #r <= 3, 'expected 1-3 results, got ' .. #r)
end)

test('search result has all fields', function()
  local r = glyf.search('A', { limit = 1 })
  assert(#r > 0)
  local v = r[1]
  assert(v.codepoint, 'missing codepoint')
  assert(v.glyph, 'missing glyph')
  assert(v.name, 'missing name')
  assert(v.source, 'missing source')
  assert(v.category, 'missing category')
  assert(v.block, 'missing block')
  assert(v.icon_set, 'missing icon_set')
  assert(v.score ~= nil, 'missing score')
  assert(v.freq ~= nil, 'missing freq')
end)

test('search empty query', function()
  local r = glyf.search('', { limit = 3 })
  assert(#r <= 3)
end)

test('search filter by block', function()
  local r = glyf.search('A', { block = 'Basic Latin', limit = 3 })
  assert(#r > 0)
  assert(r[1].block == 'Basic Latin')
end)

test('search filter by category', function()
  local r = glyf.search('A', { category = 'Lu', limit = 3 })
  assert(#r > 0)
  assert(r[1].category == 'Lu')
end)

test('search filter by source', function()
  local r = glyf.search('A', { source = 'unicode', limit = 3 })
  assert(#r > 0)
  assert(r[1].source == 'unicode')
end)

test('search filter by range', function()
  local r = glyf.search('A', { range = { 0x41, 0x5A }, limit = 3 })
  assert(#r > 0)
  for _, v in ipairs(r) do
    assert(v.codepoint >= 0x41 and v.codepoint <= 0x5A)
  end
end)

test('search filter by icon_set', function()
  local is = glyf.icon_sets()
  if #is > 0 then
    local r = glyf.search('', { icon_set = is[1].code, limit = 2 })
    -- may return 0 results depending on data, just verify no crash
  end
end)

test('search combined filters', function()
  local r = glyf.search('A', { block = 'Basic Latin', category = 'Lu', limit = 2 })
  assert(#r > 0)
  assert(r[1].block == 'Basic Latin')
  assert(r[1].category == 'Lu')
end)

test('search unknown block errors', function()
  local ok, _ = pcall(glyf.search, 'A', { block = 'NonexistentBlock' })
  assert(not ok)
end)

test('blocks returns all blocks', function()
  local b = glyf.blocks()
  assert(#b > 300)
  assert(b[1].name)
  assert(b[1].start)
  assert(b[1]['end'])
end)

test('block_of ascii', function()
  assert(glyf.block_of(0x41) == 'Basic Latin')
end)

test('block_of emoji', function()
  assert(glyf.block_of(0x1F600) == 'Emoticons')
end)

test('block_of unassigned', function()
  assert(glyf.block_of(0x110000) == nil)
end)

test('categories returns all categories', function()
  local c = glyf.categories()
  assert(#c >= 27)
  assert(c[1].code)
  assert(c[1].desc)
end)

test('sources returns sources', function()
  local s = glyf.sources()
  assert(#s > 0)
end)

test('icon_sets returns icon sets', function()
  local is = glyf.icon_sets()
  assert(#is > 0)
  assert(is[1].code)
  assert(is[1].desc)
end)

test('lookup_name known', function()
  assert(glyf.lookup_name(0x41) == 'LATIN CAPITAL LETTER A')
end)

test('lookup_name unknown', function()
  assert(glyf.lookup_name(0x110000) == nil)
end)

test('category_of known', function()
  assert(glyf.category_of(0x41) == 'Lu')
end)

test('category_of unknown', function()
  assert(glyf.category_of(0x110000) == nil)
end)

test('lookup known', function()
  local e = glyf.lookup('0x41')
  assert(e)
  assert(e.name == 'LATIN CAPITAL LETTER A')
end)

test('lookup unknown', function()
  assert(glyf.lookup('0x110000') == nil)
end)

test('frecency_path nonempty', function()
  assert(#glyf.frecency_path() > 0)
end)

io.write(string.format('\ntest result: %s. %d passed; %d failed\n', failed == 0 and 'ok' or 'FAILED', passed, failed))
if failed > 0 then
  os.exit(1)
end
