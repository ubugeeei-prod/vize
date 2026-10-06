# Compact collections all-target Clippy repair

This decision is paired with [the comment on CI roadmap issue #6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6006142235).

## Retained failure

The focused `compact-collections` workflow run [37386300391](https://github.com/ubugeeei-prod/vize/actions/runs/37386300391), job [112020300720](https://github.com/ubugeeei-prod/vize/actions/runs/37386300391/job/112020300720), checked source `145588f1b43da0eb59b410f6c3e6141ef3140988`. Its two affected files are byte-identical at actual main `40a1243f54ee7a95fe593d0504448d957adc227b`.

The existing library tests passed 444 Croquis and 251 control-flow cases. The following all-target Clippy command failed with exit 101: two helper `Option::unwrap` calls, three standard `format!` calls, and two `ToString::to_string` calls. The subsequent WASM check did not execute. Passing library tests do not establish a successful all-target baseline.

The retained complete log is 208,239 bytes with SHA-256 `30ca074cc8e5c631e8b16e11f408c22d5f46123c15dec1545938983be2e8f29c`. The immutable failure custody record has SHA-256 `7562907c4b2bf5a7d8a6a368fe4e20f78843c3c5e94270c273b778df89b6545c`.

## Narrow correction

Use the existing L0 `cstr!` formatting macro and `ToCompactString` trait in the two integration-test files. Keep every original formatting specification, complete debug comparison, dump error message, authored source, fixture and test registration. No dependency or production implementation changes are needed.

Before a helper consumes a discovered source offset, assert that the original fixture selector exists. The following optional offset default is unreachable on a missing selector because the assertion already fails; it does not accept a missing fixture span. The original complete reactivity-loss assertions remain intact.

The fourteen canonical dump pages, all grammar cases, and all six snapshot-reassignment laws retain their original input and expectations. No new lint allowance, policy change, source cap, CI schema, workflow stage or benchmark change is introduced.

## Qualification and delivery

Preparation starts from literal actual main `40a1243f54ee7a95fe593d0504448d957adc227b`. Rustfmt and diff checks are local source checks only. The repaired source has not yet executed Clippy, Rust tests or the WASM recipe; do not transfer the historical library successes to this source.

After the current v0.434 publication hold is lifted, use one small conventional PR and the existing Actions commands, preserving their order:

```sh
cargo test --locked -p vize_croquis -p vize_croquis_cf --lib
cargo clippy --locked -p vize_croquis -p vize_croquis_cf --all-targets -- -D warnings
```

The unchanged workflow's following WASM command and ordinary source checks must also pass. Keep the branch private during the hold. Track the protected queue, signed actual merge and a subsequent verified release separately; source preparation is not delivery or performance evidence.

## Current configured formatter correction

The first published source `6c2d5a4748be08cb2e87aad9d4a559654f134443` failed only its ordinary Rust formatter job [112050933715](https://github.com/ubugeeei-prod/vize/actions/runs/37395677645/job/112050933715). The complete raw log is 39,669 bytes, SHA-256 `dfd47462e6979cfce19e7949f818c63aeeb921f6d63c879131fb6a63ef585277`. The workspace uses Rust edition 2024, so the earlier local edition-2021 check was insufficient. Move only the existing `ToCompactString` import before the other L0 imports, preserving all code tokens and every original law. This is paired with [the current issue decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6006879717).

Fresh exact-source Actions remain required. The first focused workflow continues without blind cancellation; source-qualified observations remain distinct. No assertion, fixture, policy, cap, production or workflow change is made.

## Delivered Security main and native Stack qualification

Security8080 is actually signed-merged as48cb1d4f, parent143c, at05:04:12Z
on2026-10-06. Official main/merge/signature were verified before the sole
bottom integration. Both whole repaired test files, all original inputs,
assertions/snapshots and both original commit author/email/date/full-body/
reporter footers remain exact. Every incoming canonical byte is conserved at
350 lines. The [paired decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6009732436)
requires the actual native Stack8092 order8073→8086→8091, genuine literal
child ancestry and the same ordered remote identity. No independent layer
auto-merge is enabled.

Historical d83616178 source cases/ten original integration laws and focused
Clippy/WASM/436+251 units/five whole graph pairs remain old-head proof only.
Protected ea185 had104×3 proof but Security/aggregate failed and the candidate
was removed. Fresh ordinary source and one existing exact-base compact
workflow, each child's whole native vectors, protected104/Rust/native/WASM,
actual signed contiguous-prefix merges and public release inclusion remain
required. No caps/goldens/workflows/schema/production/profile inputs change,
and no historical green or queue entry is actual completion.
