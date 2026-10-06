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
start; this repair does not relocate it with a sorted block.

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

Twenty whole-file controls retain the original report, two historical inputs,
sorting, blank separation, multiple/multiline/pseudo-opener comments, raw-text
negatives, custom blocks, prologue/trailing content, BOM/LF/CRLF/CR and the
existing #6694/#3346 layout fixtures. Rust compares complete first/second/third
outputs and actual `changed` truth. The source-built CLI control retains 100
complete process captures: read-only check, three writes and final read-only
check. Full current bytes and fixed points are required, without recapture or
either-output acceptance.

These controls are prepared, not executed proof. Current Actions, all existing
source/native/legacy controls, protected instruction gates, actual signed merge
and installed release verification remain mandatory. No performance claim or
complete formatter-history adoption is made. The issue decision is paired in
[comment 6010378576](https://github.com/ubugeeei-prod/vize/issues/7877#issuecomment-6010378576)
and the central record in this same source change.
