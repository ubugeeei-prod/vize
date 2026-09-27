# Formatter differential fixtures

Two repository-authored SFC regressions from #6882 execute through the minimal
shared #6891 helper. `App.vue` files are authored inputs; `.expected.txt` files
preserve the exact existing Rust assertion references, LF and final newline.
The manifest records originating fixes and source witnesses. Exact candidate outputs match both references on all three passes. Raw records
in `capture/` identify Release run 36267966910, artifact 10915050776 and baseline
revision `f59e69c38ecbead394ba30f0fdacb5f9c1b9fd04`; publication was not confirmed
at capture. This is a candidate baseline observation, not branch CI execution
or native acceptance. The capture report's manifest digest identifies the
preactivation metadata; authored input and reference hashes did not change.

The CLI test uses only this checkout's `target/ci/vize`, after the existing
Actions source build writes its exact HEAD/binary-SHA build receipt. It runs
`fmt --no-config --write App.vue` in an isolated temporary workspace, comparing
all bytes for passes 1/2/3. Every attempt retains input/output SHA and raw
bytes, exit/signal/stdout/stderr. Config defaults are recorded separately;
`--no-config` isolates both config and ignore loading. A mismatch fails without
rewriting expected bytes. No PATH, cargo-run or skipped-pass fallback exists.

Pure checks (no CLI invocation):

```sh
node --test tests/tooling/differential-formatter.test.mjs
```

Actual source-built regression gate, after the existing CI build/receipt step:

```sh
node --test tests/tooling/differential-formatter-cli.test.mjs
```

Raw observations are written to `target/differential/formatter.json` before
asserting the gate. Both cases remain in the report on failures. Native
formatter is explicitly unsupported 2; paired comparisons, nativeHandled and
nativeEquivalent are zero. A matched reference is a legacy regression result,
not a native acceptance claim or completion of #6891/#6882.

## Existing corpus membership

The two authored inputs deliberately join global L2/L3 remarks, Canon
projection, production reach and repository-root SSR/Patina sweeps. Output
artifacts never use `.vue` suffixes. The sibling `differential` directory does
not enter `_projects` resident tuples, TS-42 roots, fuzz globs or the canonical
146-project registry.

Official L2/L3 generators and normal-mode checks have passed on the actual
fixture branch using warmed targets. L2: 445 files, 1028 remarks (156 applied,
872 missed); L3: 445 files, 150 remarks (15 applied, 135 missed). Exactly two
membership rows in each baseline and two applied L2 rows belong to these new
inputs. Removing only those new-input rows restores each old baseline SHA;
the backlog changes only its count summary. Canon's feature-gated check passed
with its unchanged 64 diagnostic comparisons. See `corpus-integration.json`
for actual commands, source identity, log hashes and old-byte proofs.

This branch's fresh source-built CLI Actions gate remains pending. Candidate
CLI capture passed separately and never creates a CI-profile receipt for the
downloaded Release artifact. Native formatter acceptance remains zero.

## Versioned filter regressions (#6845)

Three additional cases select Vue 2, Vue 2.7 and Vue 3 through a pinned
`vize.config.json`. Their physical inputs are `App.vue.txt` with explicit Vue
kind/runtime metadata; the runner writes real `App.vue` inputs in its temporary
workspace and executes `fmt --config vize.config.json --write App.vue` for all
three passes. The new references were verified through the public Rust API;
source-built CLI capture remains pending and is recorded separately from the
two candidate captures above. Five cases are now planned; native formatter is
unsupported for all five, with no paired or native-equivalent credit.

The Vue 2 CRLF case retains an independently reproduced preexisting `CRCRLF`
layout ending. It proves exact output and filter preservation without claiming
canonical newline output. The separate repair remains a recorded TODO in the
[filter decision](../../../../docs/davinci/decisions/2026-09-27-glyph-vue2-filters.md).
Global corpus membership and the two old `.vue` inputs remain unchanged.
