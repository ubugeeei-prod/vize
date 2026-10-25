# Original-owning static JSX fragments

Issue: #10522. Extends the opt-in native JSX markup view from #10484 and
#10496. This is producer admission for shared canonical consumers, not a
configured-driver migration or a module-output backend.

## Ownership and semantics

Static `<>...</>` fragments transparently publish their admitted children into
the existing canonical Region. No authored HTML element or backend-specific
anchor operation is added. Preparation validates the original opening/closing
records and all descendants before any root publishes canonical markup.

`JsxMarkupFile::fragment_boundaries()` borrows the original same-owner
`JsxNode` records in authored order. Each delimiter keeps its original parent
fragment, source, span, unit and scope. The iterator adds no retained buffer,
parser, AST walk, or producer field. `JsxMarkupFile` still has the same size as
its original `JsxFile` owner. Authored fragments already had original records;
larger authored inputs naturally contain more of those existing records.

Component/foreign-namespace, dynamic child/attribute, spread, special attribute,
entity and meaningful multiline-text boundaries remain explicit refusals.
An incomplete original File never grants this native marker. A later refusal
cannot expose canonical nodes from an earlier valid module root. The historical
public `JsxMarkupIssueKind::Fragment` variant is retained for API compatibility.

## Independent witnesses

The fixture records the actual pinned JSX transform 2.0.1, Babel 7.29.0,
runtime-dom 3.5.35 and independently executed DOM harnesses 20.11.2/21.0.4.
Unknown harness versions require their own complete record. Both recorded
harnesses produce identical normative packets.

The actual transform's VNodes independently establish wrapper-free semantics.
The complete mounted DOM includes both empty text anchors for every fragment,
including empty and nested fragments. A separate backend realization of the
native semantic nodes uses checked original boundary records to reinsert those
anchors and must equal that full mounted DOM. No mounted node is filtered out.

Independent Babel ranges cover every canonical element/text and both fragment
delimiters, including fragment nesting. The native law compares 22 complete
vectors as JSX and TSX (44 cases), complete File facts, every original syntax
record and its references, and original Program-body pointer identity.
Additional laws cover malformed/unclosed/mismatched fragments, LF/CRLF,
astral characters, exact UTF-16 positions, a nonzero embedded source origin,
owner moves and foreign-owner distinction. All 212 L2 library tests pass;
production library/probe Clippy and all six storage policy tests pass.

A separate registry-only 0.563.0 consumer reproduces the published gap:
11 static-fragment vectors return the explicit `Fragment` refusal and retain
zero canonical nodes; the compact control succeeds. This is distinct from the
new source qualification and from the published producer's separate 16-case
acceptance receipt.

## Source-phase cost

The adjacent JSON retains every sample, exact source/executable hashes, process
inventories and separate instrumented buckets. The unchanged parent executable
is tied to #10496's qualified source. Relevant L0/L1/L2/parser and Cargo inputs
are byte-identical between that source and this change's base.

The local M2 Max probe alternates nine 300-round samples at each size. No
competing compiler/native benchmark was detected before each paired sample;
the other native lanes held their work. Ordinary macOS/browser background
processes remain. These are local capability costs, not isolated-machine or
end-to-end product performance claims.

| 256 children, 257 semantic nodes | Canonical publication | Boundary query | Shared analysis |
| -------------------------------- | --------------------: | -------------: | --------------: |
| Parent compact                   |             17.782 µs |            n/a |        1.417 µs |
| Current compact                  |             17.689 µs |       1.348 µs |        1.398 µs |
| Current layout                   |             22.886 µs |       1.667 µs |        1.408 µs |
| Current 256 fragments            |             24.386 µs |       2.168 µs |        1.397 µs |

The fragment input retains 512 original delimiter witnesses and the same
257 semantic nodes. Boundary enumeration is measured separately from canonical
publication and analysis. The compact difference is not a speedup claim.
The three instrumented 100-round profiles are excluded from timing medians.
