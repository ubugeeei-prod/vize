# Original formatter fixture-gate assessment

Paired with [#6882](https://github.com/ubugeeei-prod/vize/issues/6882).
This assesses its original historical input/expected-output fixture scope.
The runtime evidence belongs to signed, actually merged source
`3e67e057348c154aab294c7178510ea6cc5c4d6d` (#7673), not to this later record.

## Original requirements and actual execution

The [immutable audit](../../../tests/_fixtures/differential/formatter-history/fix-history-audit.json)
retains the original `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943` population:
87 nonmerge area commits, 56 fixes and 150 requirements. Its eight unchanged
manifests contain 300 original plans with complete input/expected assets,
minimum distinct arms, source witnesses, retained laws and supersession proof.
All original requirements are accounted for; no missing arm or unresolved
requirement was found. The 54 semantic fixes, one test-reference maintenance
fix and one engineering fix all remain in the denominator. Engineering fix
`0e2a5bc223bc730ecc70ad71eb064d76e92f781c` (#6455) introduced no authored public
input; its checked scanners, moved tests and strict Clippy contract are retained.

[Protected Check 37164617448](https://github.com/ubugeeei-prod/vize/actions/runs/37164617448)
completed successfully on that literal merge commit. Its four Rust workers and
required reports passed. Musea and Nuxt also succeeded. Fresh REST metadata,
complete artifact ZIP hashes/CRC and the actual recorded result bytes were
independently checked; this assessment runs no formatter binary or new build.

| Contract                     | Accepted-source evidence                                                                                                                                                                | Verified result                                                                                                                              |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Eight original API manifests | Artifact [11288877349](https://github.com/ubugeeei-prod/vize/actions/runs/37164617448/artifacts/11288877349), SHA256 `4ba988fc83366935a900feb855f99f155a243ce9b536d3a5c5304f0bd842b1aa` | All 300 unique plans, 852 formatter calls and 300 complete options probes.                                                                   |
| Complete API observations    | The same artifact's eight reports                                                                                                                                                       | 276 full-byte successes with three chained fixed-point passes each, 21 typed errors and three one-call internal observations; no failed row. |
| Observer provenance          | Actual executable, Cargo JSONL/stderr and build receipt in the same artifact                                                                                                            | Source tree, lockfile, observer source, frozen executable and complete build-log hashes join to literal `3e67`.                              |
| Historical CLI behavior      | Artifact [11289546120](https://github.com/ubugeeei-prod/vize/actions/runs/37164617448/artifacts/11289546120), SHA256 `4156380f66a72b2f506d25db81dd63595d4d04286ab3510ad9e6c63cb3ffc72f` | Five scenarios, 20 check/dry/write/recheck calls; exact arguments, streams, statuses, file chains and standalone source-built CLI receipt.   |
| Original Rust laws           | Protected worker jobs 111326592324, 111326592367, 111326592377 and 111326592395                                                                                                         | All 118 law references resolve to an actual PASS, covering 110 unique functions; eight preserved-content functions pass separately.          |
| Engineering control          | Protected Rust job 111324973050                                                                                                                                                         | Strict workspace Clippy succeeds; current Rust 1.98 is distinguished from the historical Rust 1.95 control campaign.                         |

The unchanged source/manifest/report validators independently rechecked all
56 fixes, 150 requirements and 300 result rows against the accepted packet.
Every full input/output/error/stream, option, pass chain and process status
matches its pinned contract. Source authority retains whole-file hashes or
the explicitly registered retained-function transitions. Fresh downloaded
worker logs match the preserved complete logs and all named PASS observations.
This is one current 300-plan batch; three chained passes are not independent
full batches. Historical repetitions and the 24-control campaign keep their
separate source qualifications in [qualified captures](./2026-10-03-formatter-history-qualified-captures.md).

## Supplemental evidence and performance

Later sorting and configuration histories do not change the original 300-plan
denominator. Accepted-source public artifact 11289156737 retains the nine-plan,
25-call [NAPI history](./2026-10-04-formatter-public-napi-history.md) and nine-plan,
27-call [public Vite+ history](./2026-10-04-formatter-vite-cli-history.md).
Their complete objects, options, errors, process streams/statuses, file effects,
addon custody and cleanup retain their independently qualified packet review.
The sorting API's 14 plans/36 calls, configuration CLI's six plans/12 calls and
malformed configuration's eight plans/16 calls remain supplemental witnesses.

[Original formatter instruction metrics](./2026-10-04-formatter-instruction-metrics.md)
actually merged through #7673. Protected artifacts 11289310675 and 11289491015
retain 104 identities, each measured identically across three executions.
The four exact formatter ceilings are 293575, 274723, 929044 and 244347;
every old ceiling and the immutable base ratchet also passed. Calibration,
final source and protected receipts remain separately qualified. This records
instruction acceptance, with no allocation, wall-clock or RSS measurement claim.

## Printer TODO and closure boundary

The genuine OXC `.print()` error arm remains an engineering TODO on
[#6847](https://github.com/ubugeeei-prod/vize/issues/6847). Its original provider
migration `78d71cbdbb9652612e99330b6e6111902b31d6e8` (#3489), catalog refs
S200–S202, introduced no authored failing input. Its signature/zod controls
are successful outputs. No original 56-fix/150-requirement row requires that
runtime arm. At accepted source `3e67`, `crates/vize_glyph/src/script/format.rs`
lines 59–62 propagate the printer error; `crates/vize_glyph/src/script.rs`
lines 162–164 retain the expression-printer optional refusal. The original
typed-error fixtures are one style and 20 JSON errors. Settings errors, injected
IR, native Doc refusals and global-unreachability claims cannot supply printer
execution credit. A genuine provider input or separately reviewed source-bound
engineering disposition is still required and is not claimed here.

No original historical fixture or execution envelope remains missing.
#6882 is eligible for closure after this assessment and its paired issue
decision pass terminal Actions and actually merge through the protected queue.
Until then it remains open. This fixture gate does not authorize a native
formatter route: native handled/equivalent/paired counts remain zero, and
whole-SFC native formatting, native equivalence, default replacement and legacy
deletion remain unfinished under their existing product gates.
