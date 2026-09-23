# P5-4b — Summary firewall progress (not accepted yet)

Contract: [phase-5-tasks.md](../phase-5-tasks.md#p5-4b--summary-firewalls-durability-and-memory-bounds).

`vize_resident::summary` now accepts P5-2 `AlphaPages` from the upstream fact
producer. The resident `sfc_summary` query reads the source revision and
tsconfig, then builds an `SfcSummary`. The declaration query returns only the
fingerprint for a named facet and declaration. Salsa backdates equal results
at both boundaries. α pages cannot contain a body or S3 code shape.

The focused `summary_firewall` test reads a prop from one dependent file and
an emit from another. Its exact Salsa event counts are:

| Revision | `sfc_summary` exec/reuse | `declaration_fingerprint` exec/reuse | dependent exec/reuse |
| --- | ---: | ---: | ---: |
| First read | 1/0 | 2/0 | 2/0 |
| Body-only edit, same α pages | 1/0 | 0/2 | 0/2 |
| Prop type change | 1/0 | 2/0 | 1/1 |

Open buffers and α page updates have LOW durability; dependency files loaded
through `open_dependency` and tsconfig have HIGH durability. The test also
pins that a local buffer edit only validates and reuses the dependency's
summary, while a tsconfig change re-evaluates that summary and backdates its
unchanged declaration fingerprint.

The resource policy is in the recorded `linux-x64-ci` preset under
`budgets.toml [resource]`: 128 summary memos, 512 declaration fingerprint
memos, and two revisions before an unused interned declaration name can be
reclaimed. `ResidentDatabase` reads those limits and sets Salsa's LRU
capacities. These entry limits cap these two memo tables, not total process
RSS; long-lived source inputs and other stage memos have separate lifetimes.

The P5-11a RSS ceiling of 337 MiB was measured on a nine-file LSP fixture
with Corsa processes included, not a synthetic 10k-file session. P5-4b stays
open until the production α exporter calls `publish_alpha`/`revise_alpha`,
the 10k-file session is sampled with the same process-tree RSS methodology,
and its peak is below a recorded 10k-file preset. The existing TS-42 corpus
checks the block artifacts; the summary path needs a corpus edit script as
part of that final acceptance.
