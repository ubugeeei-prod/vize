# Picomatch single-branch extglob source remediation

The production audit found
[GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf)
(CVE-2026-112366, high): `picomatch <=4.0.7` backtracks exponentially on
single-branch repeated extglobs such as `+(*)` or `+(*a)`, which
`analyzeRepeatedExtglob` in `lib/parse.js` skips. npm has no patched release;
4.0.7 is the latest. The root lock resolves two affected lines, `2.3.2`
(through `anymatch` and `micromatch`) and `4.0.7` (everything else).

## Patch

Upstream has no merged fix. Both registry packages carry the same
`lib/parse.js` backport of the **open, unmerged**
[picomatch PR #266](https://github.com/micromatch/picomatch/pull/266) at head
`cad55d062b063e559913b6b0b368d7bb96c33f42`, applied without changes to
`patches/picomatch@4.0.7.patch` and `patches/picomatch@2.3.2.patch` (the v2
parser has the same structure, so only the hunk offsets differ).

A single-branch body with at least one wildcard `*` and otherwise only literals
and `?` compiles once: `+(X)` becomes `(?:X)` and `*(X)` becomes `(?:X)?`. Every
part of such a body stays inside one path segment, so two adjacent matches are
again one match and the language is unchanged. A single-branch body that mixes a
wildcard star with a slash, bracket, brace or nested group is literal text, as
`+(a|aa)` and `+(*|?)` already were. `maxExtglobRecursion: false` keeps the old
output. The package versions remain truthfully `2.3.2` and `4.0.7`.

Measured on macOS Node 24 with an input that does not match: the original
`+(*a)` takes about 0.25 s at 24 characters and doubles with each character;
the patched parser finishes 41 to 66 character inputs in well under 1 ms.

Multi-branch bodies where every branch has a wildcard, such as `+(*a|*b)`, are
outside this advisory and the upstream PR. They still backtrack. This repair
does not claim them.

## Reachability

No production path hands picomatch a pattern an attacker controls. Vize's own
direct use (`@vizejs/vite-plugin` compiler scopes) compiles patterns from the
project's Vize config with `noext: true`, so extglobs are not parsed at all.
The transitive consumers (`unplugin`, `unplugin-utils`, `tinyglobby`, `fdir`,
`@parcel/watcher`, `@vue/language-core`, `tsdown`, `nuxt`, `vue-router`,
`anymatch`, `micromatch`) take their globs from build configuration, watcher
ignore lists or file-system routing, all written by the project developer. The
patch is still applied: the audit gate must not depend on that analysis.

## Proof

`tools/commands/ci/npm-audit.ts` runs `verifyInstalledPicomatch` on every
audit. It requires:

- workspace and lock `patchedDependencies` for exactly the two versions, with
  the reviewed patch paths and their SHA-256 hashes, and both patch files on
  disk matching those hashes;
- exactly the two registry packages with pinned integrity, only the patched
  snapshots, and every lock reference to picomatch (snapshot edges, peer
  suffixes such as `fdir@6.5.0(picomatch@...)`, importers) naming a patched
  copy; aliases, overrides and audit suppressions are refused;
- every installed picomatch visible from an importer, the virtual hoist or a
  store entry's sibling links to have the full patched package digest, and to
  pass the extglob laws (compiled shapes, timed pathological inputs, preserved
  matches and the opt-out).

`assertAuditReport` recognizes a failing audit only when its whole actionable
set is this one advisory on `2.3.2` and `4.0.7`, with no patched version,
production paths ending in `>picomatch` and consistent counts. Any other
finding, a published patched version, a new version or an altered source
remains fatal. A clean audit still runs the proof, so the patch cannot drift.

Copies bundled inside other packages (for example the `picomatch` entry inside
`@voidzero-dev/vite-plus-core` and `@stencil/core`) are not in the pnpm graph
the audit reads and are outside this proof.
