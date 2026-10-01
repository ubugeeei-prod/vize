"""Replay L0 integration with preflight validation and buffered file updates."""
from pathlib import Path
import json
import os
import re

root = Path(__file__).resolve().parents[3]
os.chdir(root)
updates = {}
originals = {}


def read(path):
    path = Path(path)
    return updates[path] if path in updates else path.read_text()


def write(path, text):
    path = Path(path)
    if path not in originals:
        originals[path] = path.read_text() if path.exists() else None
    updates[path] = text


def require(path, text):
    if read(path).count(text) != 1:
        raise SystemExit('Expected one replay pattern in ' + str(path) + ': ' + text)


def validate_state():
    lib = read('davinci/vize_davinci/src/lib.rs')
    for name in ('fact', 'key', 'pass'):
        if 'pub use vize_l0::' + name + ';' not in lib:
            raise SystemExit('Partial L0 integration: missing compatibility export ' + name)
    checks = {
        'davinci/vize_davinci/src/lib.rs': 'pub use vize_l0::assert_dump_snapshot;',
        'davinci/vize_l0/src/diag.rs': 'pub mod verify;',
        'davinci/vize_l0/src/dump.rs': 'pub use collector::{Collector as DumpRuntime, Page as DumpPage};',
        'davinci/vize_l0/Cargo.toml': 'vize_l0_derive.workspace = true',
        'davinci/vize_l0_derive/Cargo.toml': 'name = "vize_l0_derive"',
        'davinci/vize_l0/src/dump/json.rs': 'pub fn push_json_string',
        'davinci/vize_l0/src/pass/observer/timing.rs': 'impl Default for TimingObserver',
        'davinci/vize_davinci/Cargo.toml': 'name = "remark_zero_cost"\n# Process-wide allocation counters must not include libtest\'s reporting thread.\nharness = false',
        'davinci/vize_davinci/tests/remark_zero_cost.rs': 'fn main()',
    }
    for path, text in checks.items():
        if text not in read(path):
            raise SystemExit('Partial L0 integration: ' + path)
    if 'vize_l0_derive' in read('davinci/vize_davinci/Cargo.toml'):
        raise SystemExit('Partial L0 integration: old derive dependency remains')
    if 'fn a_detached_remark_path_allocates_nothing()' in read('davinci/vize_davinci/tests/remark_zero_cost.rs'):
        raise SystemExit('Partial L0 integration: allocation law still uses libtest')
    for path in json.loads(read('tools/support/levels/l0-runtime-moves.json')).values():
        if re.search(r'\bvize_davinci::|crate::diagnostic\b|crate::witness\b', read(path)):
            raise SystemExit('Partial L0 integration: old source namespace in ' + path)
    if 'pub fn push_json_string' in read('davinci/vize_davinci/src/dump/feed.rs'):
        raise SystemExit('Partial L0 integration: JSON helper still owned by feed')
    for name in ('diagnostic', 'witness'):
        if 'pub mod ' + name + ';' in lib:
            raise SystemExit('Partial L0 integration: old diagnostic module remains')
    for p in Path('docs').rglob('*.md'):
        for link in re.findall(r'(?<=\]\()[^)]*', read(p)):
            if link.startswith(('https:', 'http:')):
                continue
            for old in json.loads(read('tools/support/levels/l0-runtime-moves.json')):
                if old in link:
                    raise SystemExit('Partial L0 integration: stale local link in ' + str(p))


if 'pub use vize_l0::fact;' in read(Path('davinci/vize_davinci/src/lib.rs')):
    validate_state()
    raise SystemExit('L0 runtime integration already fully applied')
for name in ('fact', 'key', 'pass', 'diagnostic', 'witness'):
    require('davinci/vize_davinci/src/lib.rs', 'pub mod ' + name + ';')
require('davinci/vize_l0_derive/Cargo.toml', 'name = "vize_davinci_derive"')
require('davinci/vize_davinci/src/dump/feed.rs', '/// Append `text`')
require('davinci/vize_davinci/src/dump/feed.rs', 'use crate::dump::remarks::push_remark_fields;')
require('davinci/vize_l0/src/dump.rs', 'pub mod collector;')
require('davinci/vize_davinci/Cargo.toml', 'vize_davinci_derive = { workspace = true }')
moves = json.loads(read(Path('tools/support/levels/l0-runtime-moves.json')))
for target in moves.values():
    p = Path(target)
    s = read(p).replace('vize_davinci::diagnostic', 'vize_l0::diag').replace('vize_davinci::witness', 'vize_l0::diag::verify')
    s = re.sub(r'\bvize_davinci(?:_derive)?\b', lambda m: 'vize_l0_derive' if m[0].endswith('_derive') else 'vize_l0', s)
    s = s.replace('crate::diagnostic', 'crate::diag').replace('crate::witness', 'crate::diag::verify')
    if target.endswith('/pass.rs'):
        s = s.replace('pub use vize_l0::pass::preserved;', 'pub mod preserved;')
    if target == 'davinci/vize_l0/src/dump.rs':
        for m in ['croquis', 'feed', 'repro']:
            s = s.replace('pub mod ' + m + ';\n', '')
        s = s.replace('pub mod collector;', 'pub mod capture;\npub mod json;\npub mod collector;')
        s += '\n// Stable aliases for the earlier foundation surface.\npub use Error as DumpError;\npub use Mode as DumpMode;\npub use collector::{Collector as DumpRuntime, Page as DumpPage};\npub use value::DumpValue;\n'
    s = s.replace('crate::dump::feed::push_json_string', 'crate::dump::json::push_json_string')
    write(p, s)
p = Path('davinci/vize_davinci/src/dump/feed.rs')
s = read(p)
start = s.index('/// Append `text`')
body = s[start:]
s = s[:start]
body = body.replace('pub(crate) fn', 'pub fn')
write(Path('davinci/vize_l0/src/dump/json.rs'), '//! JSON escaping shared by dump observers and host feeds.\n\nuse core::fmt::Write as _;\nuse vize_l0::String;\n\n' + body)
s = s.replace('use crate::dump::remarks::push_remark_fields;', 'use vize_l0::dump::json::push_json_string;\nuse vize_l0::dump::remarks::push_remark_fields;')
write(p, s)
p = Path('davinci/vize_l0/src/dump/remarks.rs')
s = read(p).replace('pub(crate) fn push_remark_fields', '#[doc(hidden)]\npub fn push_remark_fields')
write(p, s)
write(Path('davinci/vize_davinci/src/dump.rs'), '//! Compatibility exports for the L0 dump contract during consumer migration.\n\npub use vize_l0::dump::{collector, page, plan, remarks, value, Dump, Error, Mode};\n\npub mod croquis;\npub mod feed;\npub mod repro;\n')
p = Path('davinci/vize_davinci/src/lib.rs')
s = read(p)
for m in ['fact', 'key', 'pass']:
    s = s.replace('pub mod ' + m + ';', 'pub use vize_l0::' + m + ';')
s = s.replace('pub mod diagnostic;', 'pub use vize_l0::diag as diagnostic;').replace('pub mod witness;', 'pub use vize_l0::diag::verify as witness;')
s += '\npub use vize_l0::assert_dump_snapshot;\n'
write(p, s)
p = Path('davinci/vize_l0/src/diag.rs')
s = read(p).replace('pub mod witness;', 'pub mod witness;\npub mod verify;')
write(p, s)
for p in Path('davinci/vize_l0_derive').rglob('*'):
    if p.is_file():
        write(p, re.sub(r'\bvize_davinci(?:_derive)?\b', lambda m: 'vize_l0_derive' if m[0].endswith('_derive') else 'vize_l0', read(p)))
for name in ['Cargo.toml', 'Cargo.lock', 'davinci/vize_davinci/Cargo.toml', 'tests/tooling/davinci-stage-dependencies.test.ts', 'tools/moon/cmd/publish_crates/main.mbt']:
    p = Path(name)
    write(p, re.sub(r'\bvize_davinci_derive\b', 'vize_l0_derive', read(p)))
p = Path('davinci/vize_davinci/Cargo.toml')
s = read(p).replace('vize_l0_derive = { workspace = true }\n', '').replace('# Proc-macro: a host build dependency (never linked into any target, so the\n# wasm32/no_std claims are untouched) - the approved std edge P2-14 audits.\n', '')
write(p, s)
p = Path('davinci/vize_l0/Cargo.toml')
s = read(p).replace('[dependencies]\n', '[dependencies]\nvize_l0_derive.workspace = true\n')
write(p, s)
p = Path('Cargo.lock')
s = read(p)
key = 'name = "vize_l0"\n'
start = s.index(key)
end = s.index('\n[[package]]', start)
block = s[start:end].replace('dependencies = [', 'dependencies = [\n "vize_l0_derive",')
block = re.sub(r'(dependencies = \[\n)(.*?)(\n\])', lambda m: m[1] + '\n'.join(sorted(m[2].splitlines())) + m[3], block, flags=re.S)
s = s[:start] + block + s[end:]
start = s.index('name = "vize_davinci"\n')
end = s.index('\n[[package]]', start)
block = s[start:end].replace(' "vize_l0_derive",\n', '')
s = s[:start] + block + s[end:]
# Keep Cargo's lexical package order after the package rename.
derive = re.search(r'^\[\[package\]\]\nname = "vize_l0_derive"\n.*?(?=^\[\[package\]\]|\Z)', s, flags=re.M | re.S)
if derive is None:
    raise SystemExit('Missing renamed derive package in lockfile')
derive_block = derive[0]
s = s[:derive.start()] + s[derive.end():]
l0 = re.search(r'^\[\[package\]\]\nname = "vize_l0"\n.*?(?=^\[\[package\]\]|\Z)', s, flags=re.M | re.S)
s = s[:l0.end()] + derive_block + s[l0.end():]
write(p, s)
p = Path('tools/moon/cmd/publish_crates/main.mbt')
s = read(p)
lines = s.splitlines()
derive = next((l for l in lines if '"vize_l0_derive"' in l))
lines.remove(derive)
i = next((i for (i, l) in enumerate(lines) if '"vize_l0"' in l))
lines.insert(i, derive)
write(p, '\n'.join(lines) + '\n')
p = Path('docs/davinci/plan/storage-inventory.tsv')
lines = read(p).splitlines()
targets = set(moves.values())
rows = [lines[0]]
for line in lines[1:]:
    fields = line.split('\t')
    name = fields[2]
    if name in targets or (name.startswith('davinci/vize_l0/') and (not Path(name).exists())):
        continue
    if name in moves:
        fields[2] = moves[name]
    if name == 'davinci/vize_davinci/src/dump/feed.rs':
        fields[6] = '10'
    rows.append('\t'.join(fields))
rows.append('infra\t-\tdavinci/vize_l0/src/dump/json.rs\t0\t0\t1\t1\t0\t0\t0\t0')
write(p, '\n'.join(rows) + '\n')
for p in Path('docs').rglob('*.md'):
    s = read(p)

    def moved_link(match):
        link = match.group(0)
        if link.startswith(('https:', 'http:')):
            return link
        for (old, new) in moves.items():
            link = link.replace(old, new)
        return link
    write(p, re.sub('(?<=\\]\\()[^)]*', moved_link, s))

# Constructor correction is retained by replay, with an actual export-law test.
p = Path('davinci/vize_l0/src/pass/observer/timing.rs')
require(p, '#[derive(Debug, Default)]')
s = read(p).replace('#[derive(Debug, Default)]', '#[derive(Debug)]')
s = s.replace('impl TimingObserver {', 'impl Default for TimingObserver {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl TimingObserver {', 1)
write(p, s)
p = Path('docs/davinci/plan/fact-alpha-schemas.md')
write(p, read(p).replace('vize_davinci::fact::alpha', 'vize_l0::fact::alpha'))
# Allocation-law isolation is part of the replayed Cargo test contract.
p = Path('davinci/vize_davinci/Cargo.toml')
require(p, '[lints]')
write(p, read(p).replace('[lints]', '[[test]]\nname = "remark_zero_cost"\n# Process-wide allocation counters must not include libtest\'s reporting thread.\nharness = false\n\n[lints]'))
p = Path('davinci/vize_davinci/tests/remark_zero_cost.rs')
require(p, '#[test]\nfn a_detached_remark_path_allocates_nothing()')
s = read(p).replace('#[test]\nfn a_detached_remark_path_allocates_nothing()', 'fn main()')
s = s.replace('//! This binary owns the process global allocator and holds a single test,\n//! so no concurrent test pollutes the counters.', '//! This is a standalone test executable (`harness = false`): libtest\'s\n//! reporting thread must not allocate inside the process-wide measured window.')
write(p, s)
validate_state()
# Prepare every temporary file before replacing any source; no git staging here.
temporary = []
try:
    for path, text in updates.items():
        if text == originals[path]:
            continue
        temp = path.with_name(path.name + '.l0-replay-tmp')
        temp.write_text(text)
        temporary.append((temp, path))
    for temp, path in temporary:
        temp.replace(path)
finally:
    for temp, _path in temporary:
        temp.unlink(missing_ok=True)
