## #6861 — Rust test phase measurements

For [#6861](https://github.com/ubugeeei-prod/vize/issues/6861), measure
workspace-test compilation and execution separately without changing gates.
See the [measurement protocol](./2026-09-27-ci-measurements.md). Validate
normal automatic CI artifacts before closing the issue; #6865 and #6868
remain separate requirements before declaring full T1 coverage.

Execute the actual workflow phase shell against a fake Cargo executable in
focused tests. Check exact build/run arguments, required TSGO environment,
fresh HTML collection, receipt fields and propagation of nonzero Cargo status.
Follow the reusable Rust caller so extraction preserves this executable proof.

Check run `36302890896` for #6899 passed at its exact head. Artifact
`10925414722` records the matching PR head `ddc5e5bf3` and tested synthetic
merge `474386f`. Compilation took 183 seconds and execution plus doctests
took 1119 seconds, both with exit code zero; the complete Rust job took
23 minutes 32 seconds. These observations establish a baseline, not a speedup.

## Shared differential fixtures

Tracked in [#6891](https://github.com/ubugeeei-prod/vize/issues/6891).
The [first formatter path](./2026-09-27-differential-formatter.md) records the
two exact regression fixtures, source-build receipt, raw comparison, deliberate
corpus membership and remaining product/T1/T2 work. Native formatter is
unsupported; this first legacy path receives zero native acceptance credit.

[CI stack replay after publication](./2026-09-27-ci-stack-resume.md) records
the main replay, preserved corpus rows, #6910 deduplication after the first
queue merges, and remaining exact-head CI evidence.
