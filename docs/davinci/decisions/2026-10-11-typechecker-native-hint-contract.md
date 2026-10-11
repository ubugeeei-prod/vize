# Native hint diagnostics in real-project evidence

Paired decision: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6110416084).

Real Project Matrix [38147644073](https://github.com/ubugeeei-prod/vize/actions/runs/38147644073)
checked release candidate `9466fdbb90bfae8803d5e57ae267b656be051721`, whose
product source is delivered `1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71`.
The core tool observer rejected the complete Hoppscotch, vue-element-admin,
vue3-antd-admin and VitePress typechecker reports because it recognized only `error:` and `warning:`.
The shipping checker also emits native TypeScript `hint:` diagnostics.

The authenticated original stdout is retained without changing a byte in
`tests/tooling/fixtures/typechecker-native-hints/`. The receipt binds the run,
candidate, artifact identity and SHA-256, stdout size and SHA-256, exit status,
file count and all three diagnostic counts. These are observed before reports,
not replacement project inputs or recaptured after expectations.

| Project           | Errors | Warnings | Hints | Checked files | Exit |
| ----------------- | -----: | -------: | ----: | ------------: | ---: |
| hoppscotch        |   1121 |       17 |   356 |           754 |    1 |
| vue-element-admin |      2 |        0 |     6 |           169 |    1 |
| vue3-antd-admin   |    779 |        0 |     5 |           176 |    1 |
| vitepress         |     92 |        0 |     2 |           120 |    1 |

The bounded observer correction accepts a hint only with its complete positive
safe-integer line, column and TypeScript code and a nonempty message. Unknown
severities and malformed hints remain fatal. Hints do not become errors or
warnings: the original counter checks, error-based exit law, exact envelope,
file order, requested/transitive coverage and program metadata checks remain.
The original error/warning producer contracts, including script parse errors,
are unchanged. The observer does not mutate any diagnostic or raw report.

The Rust divergence collector and its compatibility mirror preserve `hint` as
its own severity. A hint in a compared Vue file enters the ordinary whole
diagnostic comparison, including its message, position and code. An unmatched
hint remains a false positive under the existing budget. The summary records
`vizeHintCount` only when nonzero; unchanged reports without hints retain their
existing whole summary and fingerprint. Existing non-Vue partitioning and the
complete pinned vue-tsc baseline contract stay unchanged. No new hint waiver,
severity normalization, skipped finding or increased budget is introduced.

The new regression first reproduced four failures under the old observers:
the first two complete original reports, a hint-only zero-error report, and the full
divergence record. The same five JavaScript laws then passed after correction,
including twelve malformed severity/location/code/message controls and invalid
counter/status controls. The complete eight-file local run passed all fifty
laws, including actual execution of both canonical Rust scripts and their five
new Rust laws. The two later reports add whole-byte regressions under the
same unchanged contract. Exact-head hosted execution remains required. Whole source fixtures, existing
diagnostic arrays, raw counters, baseline inputs and all performance ceilings
remain untouched.

This fixes an evidence contract, not the remaining real-project findings.
In the initial completed shards, vue-native-core, docsify and wakapi separately
failed native selection of their generated zero-input TypeScript projects.
Lint, LSP and typecheck parity failures belong to their own repair lanes.
The complete 22-shard artifact review found only these four hint failures and
four zero-Vue project failures in core-tools (petite-vue is the fourth).
Exact-head Actions, the full protected queue, actual
merge, a fresh complete release matrix and public release acceptance remain
unfinished. The local observer laws provide no performance or release credit.
