# Cross-file browser diagnostic origins (#7907)

The original [report](https://github.com/ubugeeei-prod/vize/issues/7907)
contains complete `WindowWidth.vue` and `PageTitle.vue` inputs. Their browser
reads are physically at 2:15 and 6:35. The opt-in cross-file producer merges
script-relative browser reads with template-relative expressions; kind-based
CLI and Doctor projection previously treated both as template offsets.

Retain primary coordinate provenance when producing each browser diagnostic.
Script and template offsets are separate domains: equal numeric offsets must
remain two findings, while duplicates within one domain still merge. Only
script-origin reads consult script client-only scopes; a template expression
cannot inherit `onMounted` protection from a coinciding script offset.

The existing CLI and Doctor adapters consume this origin. Script coordinates
still use the existing normal-script-then-setup virtual layout when both
blocks exist, independent of their physical order. Only explicitly stamped
primary offsets change projection; other diagnostic kinds and related-file
mapping keep their current behavior. Messages, severity, suggestions,
identifier matching, default non-cross-file lint and SSR classification stay
unchanged. Whether non-immediate `watch` should be classified as SSR is the
separate [#7908](https://github.com/ubugeeei-prod/vize/issues/7908) question.

`vize_croquis_cf` is an experimental Rust crate. Its diagnostic has a new
`primary_source` field and `DiagnosticSource` enum, plus a builder; both old
constructors choose `Unspecified`. This is an explicit experimental Rust
shape change, not a serialized public-output schema change. Stable product
entrypoints remain the same. No new parse, pipeline stage, native migration,
IPC, serialization or performance claim is introduced.

## Corpus and qualification

`crates/vize/tests/fixtures/issue-7907/source.json` retains the full original
body hash, verified GitHub author `ubugeeei` / `71201308`, and both original
heredocs including their terminal LF (113 and 227 bytes). Nine authored
controls cover template-only and bound-attribute reads, equal offset domains,
reversed block order, separated normal/setup scripts in both physical orders,
CRLF/Unicode, mounted scope collision and identifier prefixes.

Expectations are authored from complete source positions and the unchanged
CLI JSON contract, never recorded from the implementation. The source-built
observer requires the strict existing build receipt and compares 22 complete
CLI JSON responses (11 enabled and 11 off controls), retaining every raw
stdout/stderr/process outcome and unchanged input hash in the existing
`target/differential/` artifact. Both enabled and off calls use the existing
incremental preset and JSON output per fixture to isolate the cross-file
contract. They retain complete original inputs but do not replay the combined
opinionated-preset/plain-output argv or its other rules in the original report.

Two Rust producer laws inspect every diagnostic field and prove cross-domain
collision survival, same-domain deduplication and client-only isolation.
The public Doctor CLI separately proves the same 11 authored byte projections,
identity and unchanged inputs; it does not claim a whole Doctor JSON oracle.
All old differential corpus and native refusal evidence remains untouched.

Private pure fixture/hash/position validation passed. Compiled producer,
public CLI/Doctor, affected Rust tests and protected full suites are pending
fresh exact-head Actions; no historical runtime proof transfers. The existing
merge queue must retain all 104 instruction gates and ratchets. Actual signed
merge and reporter attribution precede the root-owned public release handoff.
This work does not block the finite 0.434 release cut.

TODO: the report's separate, unprovided `uncaught-error` example still needs
its own exact original fixture and source-domain qualification. This browser
slice does not claim every cross-file rule's coordinate mapping is repaired.
