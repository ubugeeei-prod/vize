from pathlib import Path
import json
import os
root=Path(__file__).resolve().parents[3]; os.chdir(root);
if 'pub use vize_l0::fact;' in Path('davinci/vize_davinci/src/lib.rs').read_text():
    raise SystemExit('L0 runtime integration already applied')
moves=json.loads(Path('tools/support/levels/l0-runtime-moves.json').read_text())
for target in moves.values():
 p=Path(target); s=p.read_text().replace('vize_davinci::diagnostic','vize_l0::diag').replace('vize_davinci::witness','vize_l0::diag::verify').replace('vize_davinci','vize_l0')
 s=s.replace('crate::diagnostic','crate::diag').replace('crate::witness','crate::diag::verify')
 if target.endswith('/pass.rs'): s=s.replace('pub use vize_l0::pass::preserved;', 'pub mod preserved;')
 if target == 'davinci/vize_l0/src/dump.rs':
  for m in ['croquis','feed','repro']: s=s.replace('pub mod '+m+';\n','')
  s=s.replace('pub mod collector;', 'pub mod capture;\npub mod json;\npub mod collector;')
  s += '\n// Stable aliases for the earlier foundation surface.\npub use Error as DumpError;\npub use Mode as DumpMode;\npub use collector::{Collector as DumpRuntime, Page as DumpPage};\npub use value::DumpValue;\n'
 s=s.replace('crate::dump::feed::push_json_string','crate::dump::json::push_json_string')
 p.write_text(s)
# Feed JSON escaping is a neutral helper; no observer needs the consumer schema.
p=Path('davinci/vize_davinci/src/dump/feed.rs'); s=p.read_text(); start=s.index('/// Append `text`'); body=s[start:]; s=s[:start]; body=body.replace('pub(crate) fn','pub fn')
Path('davinci/vize_l0/src/dump/json.rs').write_text('//! JSON escaping shared by dump observers and host feeds.\n\nuse core::fmt::Write as _;\nuse vize_l0::String;\n\n'+body)
s=s.replace('use crate::dump::remarks::push_remark_fields;', 'use vize_l0::dump::json::push_json_string;\nuse vize_l0::dump::remarks::push_remark_fields;'); p.write_text(s)
p=Path('davinci/vize_l0/src/dump/remarks.rs'); s=p.read_text().replace('pub(crate) fn push_remark_fields', '#[doc(hidden)]\npub fn push_remark_fields'); p.write_text(s)
Path('davinci/vize_davinci/src/dump.rs').write_text('//! Compatibility exports for the L0 dump contract during consumer migration.\n\npub use vize_l0::dump::{collector, page, plan, remarks, value, Dump, Error, Mode};\n\npub mod croquis;\npub mod feed;\npub mod repro;\n')
p=Path('davinci/vize_davinci/src/lib.rs'); s=p.read_text()
for m in ['fact','key','pass']: s=s.replace('pub mod '+m+';', 'pub use vize_l0::'+m+';')
s=s.replace('pub mod diagnostic;', 'pub use vize_l0::diag as diagnostic;').replace('pub mod witness;', 'pub use vize_l0::diag::verify as witness;')
s += '\npub use vize_l0::assert_dump_snapshot;\n'; p.write_text(s)
p=Path('davinci/vize_l0/src/diag.rs'); s=p.read_text().replace('pub mod witness;', 'pub mod witness;\npub mod verify;'); p.write_text(s)
# Derive code always targets the canonical type owner.
for p in Path('davinci/vize_l0_derive').rglob('*'):
 if p.is_file(): p.write_text(p.read_text().replace('vize_davinci_derive','vize_l0_derive').replace('vize_davinci','vize_l0'))
for name in ['Cargo.toml','Cargo.lock','davinci/vize_davinci/Cargo.toml','tests/tooling/davinci-stage-dependencies.test.ts','tools/moon/cmd/publish_crates/main.mbt']:
 p=Path(name); p.write_text(p.read_text().replace('vize_davinci_derive','vize_l0_derive'))
p=Path('davinci/vize_davinci/Cargo.toml'); s=p.read_text().replace('vize_l0_derive = { workspace = true }\n','').replace('# Proc-macro: a host build dependency (never linked into any target, so the\n# wasm32/no_std claims are untouched) - the approved std edge P2-14 audits.\n','');p.write_text(s)
p=Path('davinci/vize_l0/Cargo.toml');s=p.read_text().replace('[dependencies]\n','[dependencies]\nvize_l0_derive.workspace = true\n');p.write_text(s)
p=Path('Cargo.lock');s=p.read_text(); key='name = "vize_l0"\n'; start=s.index(key); end=s.index('\n[[package]]',start); block=s[start:end].replace('dependencies = [','dependencies = [\n "vize_l0_derive",');s=s[:start]+block+s[end:]; start=s.index('name = "vize_davinci"\n');end=s.index('\n[[package]]',start);block=s[start:end].replace(' "vize_l0_derive",\n','');s=s[:start]+block+s[end:];p.write_text(s)
# Publish derive before L0, keeping dependency order unchanged for other crates.
p=Path('tools/moon/cmd/publish_crates/main.mbt');s=p.read_text();lines=s.splitlines();derive=next(l for l in lines if '"vize_l0_derive"' in l);lines.remove(derive);i=next(i for i,l in enumerate(lines) if '"vize_l0"' in l);lines.insert(i,derive);p.write_text('\n'.join(lines)+'\n')
# Replace the former stub rows by the existing implementation rows.
p=Path('docs/davinci/plan/storage-inventory.tsv'); lines=p.read_text().splitlines(); targets=set(moves.values()); rows=[lines[0]]
for line in lines[1:]:
 fields=line.split('\t'); name=fields[2]
 if name in targets or (name.startswith('davinci/vize_l0/') and not Path(name).exists()): continue
 if name in moves: fields[2]=moves[name]
 if name == 'davinci/vize_davinci/src/dump/feed.rs': fields[6]='10'
 rows.append('\t'.join(fields))
rows.append('infra\t-\tdavinci/vize_l0/src/dump/json.rs\t0\t0\t1\t1\t0\t0\t0\t0')
p.write_text('\n'.join(rows)+'\n')
