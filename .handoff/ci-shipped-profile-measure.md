# Handoff: ci/shipped-profile-measure (2026-11-19, fleet paused)

Remove this file before merging. No PR has been opened yet.

## What it is

- `.github/workflows/shipped-profile-measure.yml`: dispatch-only, one runner. It has no PR, push or merge-queue trigger.
  - Inputs: `base_sha`, `head_sha` (40 hex characters each) and `twins` (boolean, default true).
  - It builds base, head, base twin and head twin, in that order, each in a fresh target.
  - Each build runs every level instruction suite (stages, formatter, SFC parsing). The benches build through `[profile.ci-opt]` with `CARGO_PROFILE_CI_OPT_LTO=fat` and `CARGO_PROFILE_CI_OPT_CODEGEN_UNITS=1`.
  - Each build also runs `cargo build --release` of `vize` and of `vize_vitrine` (`--lib --features napi,legacy`) and records their bytes.
- `tools/benchmarks/scripts/release-measure.ts` renders the report (`summary.md`: moved rows with twins, "N identical", A/A, sizes) as the job summary and as artifact `release-measure-<run_id>`. Its tests are in `release-measure.test.ts`.
- The workflow catalog (`README.md`) is regenerated; it lists the workflow as 03.31. The `workflow-layout` test and `vp check` pass.

## Usage

- After merge: `gh workflow run shipped-profile-measure.yml -f base_sha=<40hex> -f head_sha=<40hex> [-f twins=false]`.
- Before merge, a dispatch only works for a workflow file that already exists on main. Branch `perf/pb-rm-validate` (95c162c15e) carries this workflow in the `level-instruction-counts.yml` slot: `gh workflow run level-instruction-counts.yml --ref perf/pb-rm-validate -f base_sha=<sha> -f head_sha=<sha> -f twins=true`.

## Validation run 44510732532 (main 86b047ae43 vs itself): BLOCKING finding

- **Sizes:** identical in all 4 builds (`vize` 65,970,072 B; `vize_vitrine` 91,976,123 B).
- **Instruction counts:** head, base twin and head twin agree on all 107 rows. The FIRST build (base) differs on 26 rows by −375 to +400 instructions (e.g. `atelier_dom_codegen_large` +312, `atelier_ssr_compile_stress-deep` +400, `sfc_parse_template_only` −1, `patina_jsx_markup_one_root` +27).
  - So the first fat-LTO bench build on a fresh runner is not reproducible. Its sizes are identical, so this is something in the bench binaries.
  - This is the same ±5 kind of noise as `sfc_parse_template_only` (item 3 of the coordinator list).
- **Next steps:**
  1. Find why the first build differs. Compare `out/base/*/measurement/binaries.json` sha256 with the twins (artifact of 44510732532). Suspects: the first build's environment (cargo git checkouts, rustup first use) or the source directory path.
  2. Until that is fixed, add a discarded warm-up build of base before the four measured sides, or require base == base twin and fail loudly otherwise.
  3. Revalidate A/A (0 moved on every pair), then open the PR with that table and arm auto-merge once green.
- The renderer's A/A line should print the per-pair move counts as it does now. It currently says "differs from the primary pair" when the base side is the noisy one.

## Delete after merge

`perf/pb-rm-validate`, `perf/pb-driver` (old screening driver; `perf/pb-va5`, `perf/pb-va11`, `perf/pb-ssr-s2m`, `perf/pb-ssr-s14`, `perf/pb-ssr-s15` are referenced by handoffs).
