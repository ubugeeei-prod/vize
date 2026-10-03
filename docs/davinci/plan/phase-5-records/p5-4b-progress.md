# P5-4b — Summary firewall progress (not accepted yet)

Contract: [phase-5-tasks.md](../phase-5-tasks.md#p5-4b--summary-firewalls-durability-and-memory-bounds).

The production imported-component publication/consumer slice is recorded in
[p5-4b-production-alpha.md](./p5-4b-production-alpha.md). The history below
describes the earlier fixture-only boundary; full production scale acceptance
remains open.

`vize_incremental::summary` now accepts P5-2 `AlphaPages` from the upstream fact
producer. The resident `sfc_summary` query reads the source revision and
tsconfig, then builds an `SfcSummary`. The declaration query returns only the
fingerprint for a named facet and declaration. Salsa backdates equal results
at both boundaries. α pages cannot contain a body or S3 code shape.

The focused `summary_firewall` test reads a prop from one dependent file and
an emit from another. Its exact Salsa event counts are:

| Revision                          | `sfc_summary` exec/reuse | `declaration_fingerprint` exec/reuse | dependent exec/reuse |
| --------------------------------- | -----------------------: | -----------------------------------: | -------------------: |
| First read                        |                      1/0 |                                  2/0 |                  2/0 |
| Body-only edit, refreshed α pages |                      1/0 |                                  0/2 |                  0/2 |
| Prop type change                  |                      1/0 |                                  2/0 |                  1/1 |

Open buffers and α page updates have LOW durability; dependency files loaded
through `open_dependency` and tsconfig have HIGH durability. The test also
pins that a local buffer edit only validates and reuses the dependency's
summary. `edit_with_alpha` updates source and α pages before a query can read
them. A plain `edit` or tsconfig change without a fresh α export produces
`StaleAlpha`; the declaration query returns no fingerprint, so dependents
cannot silently accept an old contract. The producer must call `revise_alpha`
after any source or config change before normal dependent results resume.

The resource policy is in the recorded `linux-x64-ci` preset under
`budgets.toml [resource]`: 128 summary memos, 512 declaration fingerprint
memos, and two revisions before an unused interned declaration name can be
reclaimed. `ResidentDatabase` reads those limits and sets Salsa's LRU
capacities. These entry limits cap these two memo tables, not total process
RSS; long-lived source inputs and other stage memos have separate lifetimes.

The P5-11a RSS ceiling of 337 MiB was measured on a nine-file LSP fixture
with Corsa processes included, not a synthetic 10k-file session. P5-4b stays
open until the production α exporter calls `publish_alpha`/`edit_with_alpha`
and refreshes after every tsconfig change,
the 10k-file session is sampled with the same process-tree RSS methodology,
and its peak is below a recorded 10k-file preset. The existing TS-42 corpus
checks the block artifacts; the summary path needs a corpus edit script as
part of that final acceptance.

## Synthetic scale probe (2026-09-26)

The new `vize_incremental` `resource_session` example retains 10,000 distinct SFC
inputs, builds every S1/S2 artifact and six-facet fixture interface, and compares
both with the clean functions. It exercises 32 body and prop-signature edits
with exact dependent execution counts: body edits execute zero consumers;
changing the prop contract executes its consumer and reuses the emit consumer.
A high-durability tsconfig update must reject all stale exports. After refreshing
all fixture exports it revisits every cold file and declaration name, comparing
against clean results after LRU eviction and intern revision collection.

`resident-resources.rs` samples three fresh optimized processes through the same
Linux process-tree RSS sampler as TS-44 (50 ms). It preserves the existing
337 MiB peak and 316 MiB idle reference caps. Each process holds its populated
database for ten seconds; idle RSS is the maximum of its final 100 nonzero
samples. The workflow retains the full measurement JSON and per-run checks.
This is an isolated database probe with synthetic alpha inputs, without the LSP
or Corsa. It does not replace the nine-file production LSP measurement or claim
that a production alpha exporter exists. P5-4b remains open pending production
export wiring, the full process-tree scale measurement and corpus summary scripts.

Command:

```sh
cargo build --profile ci-opt -p vize_incremental --example resource_session \
  --config 'profile.ci-opt.inherits="release"' \
  --config 'profile.ci-opt.lto="thin"' \
  --config 'profile.ci-opt.codegen-units=16'
rust-script tools/commands/davinci/resident-resources.rs \
  --server target/ci-opt/examples/resource_session --files 10000 --edits 32 \
  --runs 3 --out /tmp/resident-resource.json
```

The scale probe also exposed an unbounded accounting event vector. Accounting
now aggregates executions/reuses per registered query ingredient as events
arrive, preserving existing TS-46 counts while storage scales with query kinds
rather than keystrokes. A 10,000-revision regression verifies one undrained row,
10,001 executions and a fresh accounting interval after draining.
