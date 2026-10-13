# Parallel source preflights

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision
shortens the source preflight that every PR source job waits for. The same
five checks still gate every build, in the same job and before source
selection.

The `Plan PR source checks` job spent 21 s of its 40-45 s in "Check Davinci
source inventories". There, five read-only checks ran one after another. The
generated-ledger step also spawned its five independent generators serially,
and it did so twice (`--write`, then `--check`).

`tools/support/compat/davinci/source-inventories.ts` now runs four
independent lanes together:

- the croquis consumer check;
- the consumer migration surface check;
- the storage policy test;
- the ledger `--write` followed by its `--check`.

Each lane keeps its own command order. Its output is replayed lane by lane, and
any failing lane fails the gate. The ledger script runs its generators
concurrently but emits their aggregates in the declared order. It stops at the
first failing generator, as before, so the written and checked bytes are
unchanged.

Locally the preflight went from 34 s to 12.5 s. The ledger `--write` and
`--check` each dropped from about 12 s to about 6 s. `source-inventories.test.ts`
covers lane failure, dependent-command ordering and output order.
`github-workflows-check-gate.test.ts` still pins the same five commands.
