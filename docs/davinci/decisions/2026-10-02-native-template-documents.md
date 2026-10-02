# Native Vue 3 template documents (2026-10-02)

Issues: [#6875](https://github.com/ubugeeei-prod/vize/issues/6875),
[#6847](https://github.com/ubugeeei-prod/vize/issues/6847).

This child layer adds a genuine L1 consumer on the independent
[source-borrowing document layout](./2026-10-02-formatting-document-layout.md).
Its document IR and printer remain separate modules. The syntax consumer accepts a retained L1 Vue 3
`ComponentParse` created by `dialect::vue3::surface::parse_component`, which uses
the native generic markup lexer. It never calls a Glyph or Armature parser and
never consumes L2 or L3. The only added crate edge is `vize_glyph -> vize_l1`;
the existing legacy formatter graph remains transitional.

`Doc` borrows text and uses arena-resident concatenation, groups, indentation,
breakable space/empty lines and hard lines. Group layout includes pending
continuation syntax, not just the group's own closing token. Cached flat widths
and reusable iterative stacks avoid recursively printing deeply nested Doc
groups. Width counts Unicode scalar values; it does not claim terminal/font
display width. Generated newlines select LF or CRLF, while source text retains
its authored line-ending bytes. Unbreakable source is never truncated.

The first supported syntax path formats plain opening-tag attribute layout.
Attribute order, quotes, value bytes and entities remain authored. Text,
comments, CDATA, processing instructions and closing tags remain verbatim;
content whitespace is not condensed. The shared L1 `VueDirectives` hook
recognizes directive names; this slice explicitly refuses them until typed
directive/embed documents exist. It does not repeat Glyph's attribute-priority
or normalization string matching and performs no semantic rewrites.

Source-slice custody is checked in the same traversal that constructs Doc:
nonempty token pieces must point to their original ordered source ranges and
cover the complete source. Foreign equal-byte source buffers are refused.
`TemplateDocument` retains a borrow of the original parse. Refusals keep the
caller's original source, diagnostics, recoveries and syntax observations.
Recovered/implicitly repaired trees, interpolation, unsupported directive
admission and nesting beyond 128 levels never become successful documents.
This checked carrier is not a sealed native product acceptance receipt.

Bounded local validation rebuilds the actual fresh-main L1 library against
retained dependencies and compiles the ordinary production modules. The lower
layer has seven independent Doc layout laws; this layer adds ten authentic
native-parser/output/refusal laws, for seventeen unique laws in the stack. Production and test strict Clippy,
Rust formatting and diff checks pass. The tests cover full output bytes,
quoted multiline values, generated CRLF versus authored LF, structural-depth
indentation, fixed points, continuation width, malformed-source retention,
foreign-source refusal and 20,000 iterative groups. These are scoped native
laws, not a whole Cargo dependency build, Actions or merge proof.

The preserved original combined-source attempt to typecheck whole Glyph
against old retained dependencies was held because that old L0 library does not contain the current main
`ImportSortGroup` and `SortImportsSetting` APIs. Its actual failure log remains
in the handoff. No dependency/cache mutation, replacement stub, waiver or
successful whole-product claim follows from the scoped law result.

Remaining work is explicit:

- Build documents from genuine typed directive and retained embedded syntax,
  including JS/TS and JSX/TSX, while preserving comment/source custody.
- Integrate all Vue dialects, authored verbatim modes, SFC blocks and the actual
  public formatter options with their own native providers.
- Attach the shared document-version/source-bound span-edit representation
  from #6876 instead of inventing a formatter-only edit authority.
- Run exact-source whole-product Actions, native versus legacy full-byte/error/
  fixed-point comparisons, and formatter-specific instruction-count gates.
- Close the complete #6882 fix-history gate before replacing any default
  product route. No existing entry point or CLI dispatch changes in this slice.

The full native formatter remains unfinished. These seventeen local laws add
no whole-product differential acceptance, history closure, performance budget,
hosted-check or merge credit.

## Canonical inventory (2026-10-03)

The native child initially carried the stale 59-line Glyph migration shard and
failed the real hosted source gate. Replaying it on the corrected Doc parent
and running the unchanged canonical generator records the actual 67-line shard,
including the L1 manifest edge and real consumer/parser-law sites. All other
generated shards and the original product source/test bytes remain unchanged.
This repair adds no product behavior or new runtime credit; fresh exact-head
Actions, full100 and protected Stack queue acceptance remain required.
