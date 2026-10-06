# Retain authored root-comment attachment

Issue [#7877](https://github.com/ubugeeei-prod/vize/issues/7877) reports that
the formatter adds a blank line between an inter-block root comment and the
template it documents. The original complete 141-byte `Example.vue` remains
unchanged in the regression corpus.

The existing SFC descriptor supplies the block ranges and each original root
gap. Recognize complete HTML comments with the parser's first `-->` boundary;
preserve a single separator when the terminal comment's authored trailing gap
has at most one logical line break. CRLF counts once. Content after a closed
comment is not a terminal comment. No second SFC parse, allocation, pipeline
stage or serialization is introduced by this classification.

The existing map still assigns each inter-block gap to the following block
before sorting. Standalone document prologue, authored blank separation,
trailing content, all block bodies, style cascade ordering, BOM and configured
line endings retain their original authority. Prologue remains at document
start. When sorting moves an attached group ahead of the first block, recover
only the terminal complete group after a blank-separated nonempty earlier
prologue. The same source-owned group must remain adjacent on subsequent
passes; a standalone prologue keeps its existing separation.

Two existing root-comment snapshots encode the same extra blank line. Their
original complete inputs and historical captured outputs remain immutable;
the old snapshot bytes are additionally preserved in the new corpus. Only
these two independently authored current snapshot expectations change. The
strict differential admission validates exact IDs, API, kind, options,
profile, original witness path/hash/spans, original input and historical output
hashes, plus the reviewed current authority hash. Current output must match
the complete corrected bytes; historical comparison remains `different` and
cannot be reported as an original byte match. The existing #7826 authority,
all 300 original carriers and every unrelated golden remain unchanged.

The preserved Rust source-witness guard binds the same two roles separately:
historical snapshots retain their original hashes, while the live snapshots
must match the corrected whole authority and current hashes. Restoring an old
wrong live snapshot is rejected. Every other source/snapshot guard stays exact.

Twenty-three whole-file controls retain the original report, two historical inputs,
sorting, blank separation, multiple/multiline/pseudo-opener comments, raw-text
negatives, custom blocks, prologue/trailing content, BOM/LF/CRLF/CR and the
existing #6694/#3346 layout fixtures. Rust compares complete first/second/third
outputs and actual `changed` truth. The source-built CLI control retains 115
complete process captures: read-only check, three writes and final read-only
check. Full current bytes and fixed points are required, without recapture or
either-output acceptance.

These controls are prepared, not executed proof. Current Actions, all existing
source/native/legacy controls, protected instruction gates, actual signed merge
and installed release verification remain mandatory. No performance claim or
complete formatter-history adoption is made. The issue decision is paired in
[comment 6010378576](https://github.com/ubugeeei-prod/vize/issues/7877#issuecomment-6010378576)
and the central record in this same source change.

The first normal source Check [37422249485](https://github.com/ubugeeei-prod/vize/actions/runs/37422249485)
at `1b2ef4a5` authentically failed: tooling shard3 found one extra LF on
the sorted historical input's second full pass, and the preserved-source
scratch test lacked the new authority/historical files. Shard1 also found
the generated formatter source inventory stale. Retain both full failed logs
as historical evidence. The successor recovers only the existing attached
comment group after the earlier prologue, copies all three exact authority
assets into the scratch fixture with drift/missing controls, and regenerates
only the changed formatter inventory shard. The synthetic validator records
the two complete historical DIFFERENT/current EQUAL comparisons explicitly.
Every original/current expected
byte remains unchanged from the independently authored repair. Fresh complete
Actions and the bounded source peer, not these source edits, qualify delivery.

This successor is paired with [comment 6010655474](https://github.com/ubugeeei-prod/vize/issues/7877#issuecomment-6010655474)
and the central record in the same change.

The independent source peer found that an after-process file read could fail
before the complete process row was retained. Record stdout/stderr/status/
signal/process error immediately after `spawnSync`; then retain after-file
bytes/hash or its read error before propagating failure. All original inputs,
current expectations, process assertions and 20/100 capture laws remain exact.
This observer correction provides no product execution or performance credit.

The source peer identified unchecked source-derived string slices against the
existing production Clippy policy. Use checked `get`/`split_at_checked` and
conservatively retain unchanged classification if a boundary is refused,
without lint allowances or expected-byte changes. This is a source compile-risk
correction; fresh hosted Clippy and native results remain required.

A comment-only accepted SFC has no following-block consumer. The source peer
found that extracting its terminal group would drop that complete comment.
Split the prologue only when a real parsed block exists; otherwise retain the
entire document prologue. Add complete two-comment/blank, raw-text-plus-comment
and BOM comment-only inverses while preserving every original first20 input
and current expected byte. The full current corpus is23 cases/115 process
captures, with unchanged whole-output/check/three-pass/status/fixedpoint laws.
Stdout/stderr are whole retained observations; elapsed CLI summaries are not
asserted as byte-stable output. No execution credit transfers from blocked heads.

The intermediate PR conflicted against advancing main and had no normal Check
for those intermediate heads. Preserve every incoming canonical line and the
complete owned decision clause while composing fresh signed main; no workflow
bypass, incoming-source substitution or expected-byte relaxation is permitted.
