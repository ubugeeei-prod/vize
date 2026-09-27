# NextTick arrow history through the actual public API

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Capture parent: `4c7730325968403a71429925767871094be22f05`.
Historical requirement: `c7c1d7c968a8528a945a57693ce092fd3872fd4d`.

## Decision and exact scope

The arrow return repair has five private `ScriptLinter` tests that assert
warning counts. Retain those assertions and preserve their exact source
bytes in a separate public API cohort. An expression arrow returning the
promise, including a parenthesized call and imported alias, must remain
clean. A bare call in a block, a `void` expression arrow, and a bare call
after a nested expression arrow must report the ignored result. The nested
case protects restoration of the surrounding statement context.

[Ten strict JSON inputs](../../../crates/vize_patina/tests/fixtures/next-tick-history/cases.json)
pair each private source with a standalone TypeScript file and an ordinary
`<script lang="ts">` SFC. The SFC prefix contains Unicode, emoji and CRLF;
its script body begins at UTF-8 byte 57. The exported alias example retains
its original export rather than changing the source for script setup.

| Existing private test suffix                  | Standalone diagnostics | SFC diagnostics |
| --------------------------------------------- | ---------------------: | --------------: |
| `valid_arrow_expression_return`               |                      0 |               0 |
| `valid_parenthesized_arrow_expression_return` |                      0 |               0 |
| `invalid_arrow_block_bare_call`               |                      1 |               1 |
| `invalid_arrow_expression_void_call`          |                      1 |               1 |
| `nested_arrow_restores_statement_context`     |                      1 |               1 |

The separate `next_tick_history` observer calls real `Linter::lint_script`
and `Linter::lint_sfc`, with Incremental preset, only
`script/valid-next-tick` enabled, English locale and full help. It snapshots
all input/configuration and ordered public `LintResult` Debug fields,
including filename, counts, diagnostic rule/severity/message, byte ranges,
full help, every label and the actual offered fix. It independently applies
each offered fix to the original input and requeries; this rule actually
offers none in all ten cases, so unchanged inputs are queried again.
Every entire observation repeats and must be byte identical before matching
its complete golden. No diagnostics or locations are reconstructed.
The existing 13 current API cases and four report cases remain unchanged.

## Actual capture and ledger preparation

The source-built, locked offline `-p vize_patina --test next_tick_history`
executable used the authorized existing target with incremental compilation
disabled; no target was cloned or copied. Rust 1.98.0 on
aarch64-apple-darwin built the selected target in 42.16 seconds, captured
ten actual goldens and then passed with `INSTA_UPDATE=no` (one test,
zero ignored). All five pairs produced the required 0, 0, 1, 1, 1 sequence.
The [capture receipt](./2026-09-27-next-tick-history-receipt.json) binds
full base source and build-input identities, the staged test/input patch,
case bytes, actual executable, target/profile/features and raw log hashes.
This is a historical local capture; later replay and Actions runs must bind
their own source and executable rather than claiming this capture was
performed at a later head.

The official consumer migration generator produced only four added rows
in the existing linter TSV shard: the new `next_tick_history` row and three
inherited missing rows (`lint_history`, `report_history`, `static_class_fix`)
at this private capture parent. The exact generated rows are retained here;
there are no generated global totals. Its `--check` validates all 19 files,
and all 16 focused consumer migration tooling tests pass. A later replay
onto the published parent must keep inherited rows unchanged and add only
the new cohort row. The capture parent is not claimed to have passed all
required CI checks.

## Remaining work

TODO: register these exact inputs and oracles with shared #6891; review the
other independent requirements and supersessions in the historical commit;
verify fresh source-built Actions and merge queue runs. These ten selected
legacy-backed observations certify no whole history row and provide zero
native handling, equivalence or paired-comparison credit. The broad
inventory at `b4f25fb6511075aa531be80d645bb0db8cc151e0` remains a separate
258-candidate title inventory; #6881 stays open.
