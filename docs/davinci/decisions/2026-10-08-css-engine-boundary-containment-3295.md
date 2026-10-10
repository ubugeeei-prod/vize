# CSS engine containment in formatter and linter consumers

The original [#3295](https://github.com/ubugeeei-prod/vize/issues/3295)
percentage inputs still crash two consumer paths outside the Atelier CSS
boundary. This change isolates those calls and uses their existing invalid-CSS
fallbacks. It does not retire the Atelier boundary or close #3295 or
[#3864](https://github.com/ubugeeei-prod/vize/issues/3864).

The read-only baseline uses source-qualified host artifact
`3390cb6044d53d655a9d64e112d2618375cff5ca`, produced by run `37717489915`.
Its Darwin arm64 addon has SHA256
`b884696038e3e6c09054b73d9532a3ab490d2cd296a5770d836561b0be245d1d`.
This is a CI artifact observation, not an installed-package or publication
claim. The relevant ten consumer/boundary source files and four dependency
lock blocks are byte-identical to genuine main
`b41811e3b33676f68a0843a693c9ed4898f2e86a`.

On 2026-10-10 the prepared change was replayed in an isolated worktree on
`26031a4fbb30a1e86511919bafc7035b08440ef3`. All six original consumer files
still match that current source before the fix. Both original Rust observer
bodies and the complete nineteen corpus files remain byte-exact. The five
focused local API laws pass with 99 complete repeated observations. The same
formatter and linter laws fail on the unchanged current consumer source, then
pass again with the fixed files restored. This is local source evidence; fresh hosted
CLI, native, protected-budget and publication qualification remains required.

The independent source PR replays the same runtime change on signed main
`4c2bebb9a586e3eaab698b41e5b58ed5f8181852`. Its thirty-seven incoming release
metadata files remain byte-exact, as do the complete original corpus and API
laws. The earlier local controls remain qualified only to `26031a4f`; the
new source head requires its own full hosted CLI, native and instruction-count
results. Queue admission and publication remain separate pending actions.

Separate finite child processes retain the complete arguments, input,
stdout, stderr and exit status. `a{opacity:abs(-50%)}` exits by SIGABRT through
`formatSfc` and CSS-only `lintPatinaSfc`, while `parseCssAst` returns the
existing guard error. The original color-component and number-folding
inputs also abort through the formatter. The under-guarded
`a{border-image-slice:abs(-50%)}` returns the existing boundary error from
`parseCssAst` after a real engine unwind, proving that artifact containment
works where the boundary is present. Healthy percentage and number neighbors
retain complete successful results. The markerless-list accessibility parser
has its own direct CSS entry and requires a separate isolated control.

Glyph contains only the actual color parse, stylesheet parse and CSS print
calls. Engine panics become its existing `StyleFormatError`; SFC formatting
then follows the existing authored-text fallback. Patina's two direct CSS
parse sites share a private helper and keep their existing invalid-CSS
fallbacks: skip CSS rules for that block and decline markerless-list evidence.
Ordinary template findings and later healthy style blocks remain observable.
The normal parse count, printer, options and authored-token preservation
policy remain unchanged. No stage, serialization or dependency is added.
The engine's existing panic hook remains visible in captured stderr.

All original CSS bytes are retained in the differential corpus, alongside
whole healthy finding and output controls. Public Rust API tests and
source-built CLI processes must compare complete outputs and formatter fixed
points. A separate hosted CLI regression cohort registers the new corpus; the existing
sixteen-case shared CLI manifest remains byte-exact and grants no new historical
execution credit. The original 300 API obligations, thirteen
historical CLI obligations and original Rust law bodies retain their pins.
Fresh exact-head Actions, protected instruction budgets and actual merge
remain required; the read-only baseline grants none of those qualifications.
The complete ninety-process CLI observer is erasable TypeScript and uses the
current typed build-receipt and SHA helpers. It belongs to the existing full T1
runtime cohort, preserving its original loops, complete result comparisons,
sixty-second process timeout and eight-MiB capture limit. It adds no release
gate, changes no historical execution credits and relaxes no performance cap.

The released [LightningCSS v1.33.0](https://github.com/parcel-bundler/lightningcss/releases/tag/v1.33.0)
still corresponds to locked Rust `1.0.0-alpha.72` and retains the percentage
panic. Unreleased [commit b0939ab](https://github.com/parcel-bundler/lightningcss/commit/b0939ab684793d2ba3ccc1a91e0f4d3ba837a708)
returns an invalid-value error inside a wider numeric-range change; this
change does not adopt that unreleased implementation. Released
[`cssparser-color 0.6.0`](https://github.com/servo/rust-cssparser/blob/d128a90e1203fc4afb959420d6dc5e24f77a3191/color/lib.rs#L306-L307)
still asserts the finite hue range and requires a different cssparser minor.
The pathological unterminated-function runtime class also remains a separate
retirement obligation. A catch boundary does not provide a runtime budget.

Published `parcel_sourcemap 2.1.1` still requires `rkyv 0.7.38`, resolving here
to `0.7.46`; LightningCSS's bundler feature requires its sourcemap feature.
[RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235.html)
requires `rkyv >=0.8.17` and describes the unsupported affected 0.7 series.
The existing no-archive-deserialization justification and audit waiver remain
unchanged. Removing `@import` bundling or undertaking a broad archive API
migration would exceed this bounded containment fix.
The 2026-10-10 crates.io index still lists LightningCSS `alpha.72` and
`parcel_sourcemap 2.1.1` as their newest published versions. The former requires
`cssparser-color ^0.5.0`; published `0.6.0` is outside that requirement. The
cached released archives independently match the current lockfile checksums,
and the released percentage parser retains its `unreachable!()` branch. No
released dependency update satisfies either retirement issue.

The same decision is recorded in [#3295 comment6052094270](https://github.com/ubugeeei-prod/vize/issues/3295#issuecomment-6052094270). Upstream repositories remain
read-only. Current release source/version pins are unchanged; installed
delivery belongs to a later independently qualified release.
