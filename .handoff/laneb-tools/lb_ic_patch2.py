"""Add fixture byte-offset reporting (mod 64) to the disasm-patched harness script."""
S = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/'
src = open(S + 'ic-patched.mjs').read()
anchor = '      fs.writeFileSync(path.join(dir, `${pkg}-${bench}.s`), lines.join("\\n"));\n'
assert src.count(anchor) == 1
extra = anchor + r'''      // Where each ladder fixture's bytes sit in the file (file offset mod 64
      // equals the virtual address mod 64 for a page-aligned segment).
      const image = fs.readFileSync(executable);
      const fixtureDir = path.join(root, "tools/benchmarks/crates/davinci_harness/fixtures");
      const offsets = {};
      for (const name of ["small", "medium", "large", "stress-deep", "stress-wide", "stress-interp"]) {
        const bytes = fs.readFileSync(path.join(fixtureDir, `${name}.vue`));
        const at = image.indexOf(bytes);
        offsets[name] = at < 0 ? null : { offset: at, mod64: at % 64 };
      }
      fs.writeFileSync(path.join(dir, `${pkg}-${bench}.fixtures.json`), JSON.stringify(offsets, null, 2));
'''
src = src.replace(anchor, extra)
open(S + 'ic-patched2.mjs', 'w').write(src)
print('ok')
