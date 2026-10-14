# Validate the expected parser recovery in untimed leaf parity

The repaired nested cache setup in [#8887](https://github.com/ubugeeei-prod/vize/pull/8887)
exposed a benchmark harness failure in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Run 38415006953, attempt 2, retained 270 successful process samples before the
first escaped-import fixture returned TS1260 and the independent shared-file
TS2304 diagnostic. Parser recovery correctly completed semantic checking, but
the generic normalizer rejected its expected stderr warning.

Mark only the existing escaped-keyword and malformed-import/export fixtures
as expecting parser recovery. For their untimed parity and parity-profile
processes, accept the exact check-failure recovery notice (single CLI to session,
or the first sharded CLI to single CLI step on two-server runs), optionally followed
by the known native session EOF or context-cancelled shutdown notice, only with exit status 1 and a
syntax diagnostic. Retain original raw stderr before adapting the result.

Run 38426550040 then reached sample 286, the existing `loader.js` JSX error
fixture. It uses the same recovery path but reports TS7026 in `loader.js`
and the independent shared-file TS2304, rather than a syntax diagnostic.
Mark this one fixture separately and require its `loader.js` TS7026 before
normalizing the known warning. Other semantic errors do not qualify.

Run 38429417207 retained the complete sample 271 TS1260/TS2304 report with
`context canceled` at session shutdown instead of EOF. The adapter permits
that exact alternative; a cancellation alone, multiple shutdown lines, or an
incomplete report still fails. The pinned Corsa process inherits stderr and
Vize closes its session on drop. Remove these shutdown alternatives when that
runtime closes recovered sessions without writing stderr.

Every complete diagnostic/program vector, base/head comparison, one/two-server
comparison, and partition assertion still runs. Timing, cold, warmup, ordinary
plants and other fixtures retain strict stderr rejection. Unknown or repeated
notices, spawn/parse failures and invalid report counts still fail. No product
behavior, timed sample or check is removed.

The regression tests reproduce the retained syntax and semantic diagnostics,
cover those refusal cases, and preserve raw evidence. The full paired workload
is verified on Actions after the harness repair.
