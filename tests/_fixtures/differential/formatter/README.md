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

Before the fixture PR is published, regenerate existing L2/L3 Folios and the
L2 generated remarks backlog with their actual tests, preserve old rows, and
verify without UPDATE. Measure Canon's exact 64 companion gate; its eligible
diagnostic rows are expected unchanged for these static/v-pre templates, but
source inspection is not a passing run. Those corpus measurements and this branch's source-built CI gate are pending.
The separately recorded candidate CLI capture has passed; it does not create
a CI-profile build receipt for the downloaded Release artifact.
