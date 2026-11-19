# Handoff: fix/lsp-art-imported-hover (PR #13413). Delete before merge.

Paused 2026-11-19 (fleet pause). PR #13413 is open, auto-merge OFF.

## State
- The fix for #13383 is done. Art variant hovers now open through the canonical native project (`ide/corsa_support/art_template.rs`).
- Fixtures:
  - `lsp/regression/art-variant-imported-hover-13383` (LSP differential manifest; the fix-title gate passes locally).
  - `lsp/fix-history/art-variant-imported-callables-13383` (ledger row 0fd57ff915 is covered).
  - Rust test `ide/diagnostics/art_variant_hover_after_diagnostics_tests.rs`.
- #13385 is not a regression. It is pinned as `lsp/fix-history/native-root-required-props-13385` (ledger row 09b4ddde76 is covered). Close #13385 when this PR merges, and close #13383 too.
- Commits: ce9a13a2ab (fix), a544ffb586 (#13385 pin), c251f18b78 (differential case, plus the PR gate fix).

## In-flight measurement runs (ci-opt `level-instruction-counts.yml`)
- Base: 44517750227 on `bench/lsp-13383-base` (main 8b09dea8b1). Canon PR #13414 also uses it, so do not cancel or re-dispatch at that SHA.
- Head: 44517582455 (a544ffb586). Its Rust sources are identical to the current head c251f18b78.

## Next steps
1. Collect both runs. Dispatch the twins at distinct test-only SHAs (base twin and head twin) and write the A/A table.
2. Release-profile `vize` size: dispatch `level-instruction-counts.yml` on `bench/levels-release-size` with `-f base_sha=8b09dea8b1 -f head_sha=<head>`. The new module adds code, so get a ruling if the release binary grows.
3. Add the PR performance observation (wall, CPU, RSS) table to the body. The `MEASUREMENTS` section is pending.
4. Locally, after a rebase: rebuild the ci binary, then run `node tests/differential/build-receipt.ts` and `tests/tooling/differential-lsp.test.ts`. The new manifest case was not executed locally because disk was under 60 GB.
5. Enable auto-merge once everything is clean. Delete this file before merge.
