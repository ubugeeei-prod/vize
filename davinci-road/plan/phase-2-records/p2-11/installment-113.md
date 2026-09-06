# P2-11 Installment 113 - S2 DOM Comments

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5796](https://github.com/ubugeeei-prod/vize/pull/5796), merged
> 2026-09-06 as `a7ab8b2a5`.

This installment gives ordinary template comments an S2 DOM output path.
Earlier production-selector work kept `comments` on the compatibility path
because S1-to-S2 lowering dropped comment nodes before the emitter could
materialize `_createCommentVNode(...)`. The S2 lane now preserves the authored
comment boundary that matters to DOM codegen and emits the same comment vnode
shape as the shipped lane.

## What changed

S2 gains an explicit comment-bearing text op rather than teaching the emitter
to recover comments from source bytes after lowering. That keeps the output
feature in the same lowered tree as adjacent text, slot and branch decisions,
and lets existing helper-order, budget and folio-span witnesses observe the
new node like every other emitted child.

The production selector now treats `comments` as projected onto S2, while
`experimental_in_tag_comments` stays off the supported surface. The distinction
is intentional: ordinary comments are authored template children; in-tag
comments are a parser extension inside an opening tag and still need their own
admission evidence.

## Evidence

`crates/vize_s1_to_s2/tests/emit_comments.rs` pins comment emission through S2
directly, including slot, fragment and branch positions. The existing folio
and hoist witnesses were adjusted so the new comment op remains visible to
span and static-lattice checks.

`crates/vize_atelier_dom/tests/davinci_s2_sfc_comment_selector.rs` compares a
comment-preserving SFC sections compile against the compatibility lane and
asserts the selected compile records one `davinci.s2_dom.files` counter. The
full emitted body is pinned exactly, including `_createCommentVNode("kept")`.

This installment does not tick P2-11. The full production-lane switch remains
open because experimental in-tag comments, unsupported option shapes and the
explicit legacy flag still require the compatibility path.
