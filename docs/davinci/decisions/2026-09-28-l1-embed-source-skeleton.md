# L1 typed-embed source skeleton

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
its [approved design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).
The maintainer explicitly requested actual code skeletons with `todo!()`
before the full native implementation. This is a bounded preparation under
that instruction; #6835/#6841 prerequisites and #6836 remain open.

The change starts on actual main
`754c0acd2c9246a534ad43f22cfc7dd5b6960e1a`. It adds `vize_l1::embed` with
independent `Shape`/`Lang`/`Grammar` records and borrowed `Embed`/`EmbedSource`
records. `embed::source` carries a borrowed decode map whose decoded offsets
are embed-relative and whose authored offsets are absolute file coordinates.
The `prepare_attribute_value` entry point explicitly panics with `todo!()`;
there is no fake successful result or legacy adapter.

Only that function expects `clippy::todo`, with the unfinished issue in its
reason. The existing workspace denial remains intact. No dependency, feature,
Cargo target, wire protocol or existing product call path changes. Current
surface-tree types and arena layout are unchanged. These declarations add no
parse, analysis, serialization or observation stage to production.

TODO before this boundary becomes usable:

- Validate source ranges and UTF-8 boundaries; implement lazy entity decoding
  and exact span correspondence, including complete coverage and wrapper laws.
- Resolve host language once per container; diagnose script language mismatch.
- Add OXC-backed embedded trees, comments and typed holes, keyed by the extracted
  L0 identity after #6833, without creating a competing node allocator here.
- Implement dialect syntax hooks, composite shape construction and the static
  L1-to-L2 pattern table after their ready prerequisites. Consumers must reuse
  the embedded syntax rather than parse again or project Croquis facts.
- Execute real default/feature/target compile and existing fidelity/runtime
  checks in Actions, then protected merge-queue suites and instruction gates.
  Source formatting alone is not compiler, byte-parity or native acceptance.

Private source review and paired issue comment precede publication. Existing
fixtures, current main's merged fixes and the separate private naming/performance
work are preserved. No issue is closed by these skeleton declarations.

Local verification: project Rust formatting, both decision-page formatting and
whitespace checks pass. Every changed/new file is below 350 lines. No Cargo,
compiler, Clippy or product-runtime execution ran locally; actual Actions
compile/tests and the protected merge queue are pending publication.
