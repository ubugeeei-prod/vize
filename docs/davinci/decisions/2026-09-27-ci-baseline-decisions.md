## #6861 — Rust test phase measurements

For [#6861](https://github.com/ubugeeei-prod/vize/issues/6861), measure
workspace-test compilation and execution separately without changing gates.
See the [measurement protocol](./2026-09-27-ci-measurements.md). Validate
normal automatic CI artifacts before closing the issue; #6865 and #6868
remain separate requirements before declaring full T1 coverage.

## Shared differential fixtures

Tracked in [#6891](https://github.com/ubugeeei-prod/vize/issues/6891).
The [first formatter path](./2026-09-27-differential-formatter.md) records the
two exact regression fixtures, source-build receipt, raw comparison, deliberate
corpus membership and remaining product/T1/T2 work. Native formatter is
unsupported; this first legacy path receives zero native acceptance credit.

[CI stack replay after publication](./2026-09-27-ci-stack-resume.md) records
the main replay, preserved corpus rows, #6910 deduplication after the first
queue merges, and remaining exact-head CI evidence.
