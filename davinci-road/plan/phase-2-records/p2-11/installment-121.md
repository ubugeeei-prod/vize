# P2-11 Installment 121 - Optimize Imports No-op

## What landed

[#5849](https://github.com/ubugeeei-prod/vize/pull/5849) removes the stale
`optimize_imports` production-selector guard after auditing that the legacy DOM
codegen never reads that adapter-only option. `CodegenOptions::optimize_imports`
now keeps the S2 DOM production selector armed, with a legacy false/true parity
witness and profiler counter witness proving the selected render bytes still
match the shipped compatibility lane. The main-line pin is `44058cef0`.

## Evidence

- `cargo fmt --all -- --check`
- `cargo test -p vize_atelier_dom compile::stage_options::tests --lib`
- `cargo test -p vize_atelier_dom --test davinci_s2_codegen_option_selector`
- `cargo test -p vize_atelier_dom --test davinci_s2_production_selector`
- `node --test tests/tooling/davinci-dom-production-boundary.test.ts`
- `git diff --check origin/main...HEAD`

## Remaining blocker

P2-11 remains open because opaque custom-element predicates and the explicit
legacy flag deletion still require the compatibility path.
