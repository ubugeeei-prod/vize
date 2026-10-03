# L3 laws for genuine selected zero-header HTML bodies

Issues: #6839 and #6840. This is a bounded fixture child of the actual
selected static HTML provider and the merged original-template L3 entry.

## Decision

The private fused L2 provider now constructs selected zero-header HTML
Element bodies containing nested Element, Text and Comment operations. The
existing `build_native_dom_file_decisions` consumes that provider's completed
`NativeTemplateView` unchanged. The new laws exercise that genuine owning
pipeline; they do not construct a neutral File or pair a separate template
with one.

The fixture lives in `tests/native_template_cases/element.rs`, registered
through ordinary `mod native_template_cases` and `mod element` declarations
below the existing `native_template.rs` integration root. It
adds no Cargo-discovered helper target, production entry, side table, tree
walk, pipeline stage or serialization. The root-only historical bounds in
[the original entry decision](./2026-10-03-l3-original-template-analysis.md)
remain attributed to their dated provider.

## Exact successful law

The full authored source is
`<template><section><span>hé</span><!--note--><br/></section></template>`.
The moved lower output, internally derived File and artifact remain the same
objects. Every DOM node borrows the exact canonical Op at its real preorder
id, with no copied tree or synthetic ids. The source, UTF-8 text and comment
body retain their original source slices.

| Id  | Actual operation | Full source byte span | Neutral and output StaticLevel |
| --- | ---------------- | --------------------- | ------------------------------ |
| 0   | section Element  | 10..61                | Dynamic                        |
| 1   | span Element     | 19..35                | Static                         |
| 2   | Text `hé`        | 25..28                | Static                         |
| 3   | Comment `note`   | 35..46                | Dynamic                        |
| 4   | br Element       | 46..51                | Static                         |

There are exactly five canonical nodes and five decision rows. The authored
Comment keeps the existing conservative Dynamic classification, including
its containing section; this law changes no classification policy.
All three Elements have HTML namespace and empty attributes/bindings. There
are no expression rows, controls, dynamic bindings or unsupported DOM facts.

The DOM root is Direct with the actual node 0. Its Array children are exactly
nodes 1, 3 and 4. Node 1 has a nondynamic Text group containing only node 2;
nodes 2, 3 and 4 have Empty children. Only node 0 is block eligible. The entire
semantic first-use dependency sequence is exactly `NativeElementValue`,
`CommentValue`, `BlockBoundary`, `NativeElementBlock`.

## Refusal and interruption laws

For each nested repeated-whitespace text, entity text and SVG child, the
provider retains only the actual constructed section/Text/span prefix with
three nodes. Its refusal remains the same `UnsupportedChild` issue through
the later original sibling and root completion attempt. The incomplete File,
original selected root children and interruption observation remain owned;
the lower view refusal prevents invocation of native analysis.

Consuming the complete original five-node Element body and then dropping the
root walk also leaves an incomplete File and an Interrupted view. Completing
an Element body cannot stand in for normal selected root completion. All
five actual operations, including the original nested UTF-8 text, Comment
and br, remain inspectable in that refused owner.

## Delivery and unfinished work

This preparation verifies source formatting and diff integrity only. Rust
execution remains fresh exact-head Actions work after the child is replayed
on the refreshed public provider. The native Stack, full merge-queue suites,
unchanged instruction gates and terminal merge are still required; cached
libraries and previous provider results grant no execution credit to these
new laws. The paired issue decision is published with that child.

The first child campaign at `22f1bf50d0` compiles successfully but rejects
the original path-attributed module in the existing module-layout gate.
The unchanged law body moves in a move-only commit; ordinary module
discovery replaces the path attribute without a gate exception or new
Cargo-discovered test target. The failed campaign remains failed, and the
corrected child requires fresh exact-head Actions.

No native output, complete module/map/runtime comparison, target or product
acceptance credit is added. Element headers, operands, decoded text,
If/For, JSX, other Vue dialects, setup semantics, L4/SFC migration and default
product replacement remain outside this fixture. The existing fix-history
and performance gates stay unchanged.
