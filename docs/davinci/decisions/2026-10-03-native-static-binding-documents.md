# Native static binding documents (2026-10-03)

Issue: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847).

The opt-in native template document consumer now accepts the first typed Vue
directive family: shorthand `:argument` and `.argument` with a static argument.
The existing L1 `VueDirectives` provider supplies `DirectivePrefix`,
`ArgSyntax::Static` and absolute argument/modifier spans. Glyph does not classify
directive spelling itself, reparse values or introduce another syntax stage.
This adds no crate edge or provider facade.

Each attribute head first becomes a checked L0 `SourceBlock` of the retained
parse's actual source. The typed prefix, static argument and modifier spans
must form the complete ordered head. Every projection must remain within that
block at UTF-8 boundaries; its Doc text borrows the original source bytes.
Foreign equal-byte token text cannot establish custody. The document continues
to borrow the caller's original `ComponentParse`, with its diagnostics and
recoveries retained.

Opening-tag layout uses the existing separate Doc printer. It preserves
attribute order, complete directive head spelling, modifier spelling, values,
quotes, entities and embedded JS/TS bytes. Only the already supported attribute
separators and whitespace around `=` are laid out. Authored multiline values
stay verbatim, while generated indentation and line breaks follow Doc options.
No expression is decoded, parsed, normalized or semantically rewritten.

Dynamic arguments, full `v-bind` spelling, event/slot and other directive
families remain explicit refusals. Interpolation, recovered trees and implicit
repairs keep their existing refusals. These are bounded consumer admissions,
not claims that other valid Vue syntax is invalid. Plain attributes and source
content retain their existing native output contracts.

Two projection laws check original pointer identity, Unicode argument/modifier
coordinates and rejection of forged spans that split UTF-8 or leave the block.
Six complete-output/refusal laws cover flat and broken heads, opaque JS/TS and
entity values, Unicode and multiline values, attribute order, fixed points,
unsupported families and foreign equal-byte heads. The existing seven Doc
printer laws and ten native template laws also pass, for 25 unique laws.

The scoped proof compiles the whole current signed-main L1 production source
and the selected ordinary native Doc modules against pinned retained dependency
artifacts. Strict production Clippy and the actual 25 laws pass. The first
fixed-point test used an unquoted `b/>` value that the existing L1 parser recovers;
its failed log is retained, and the supported-input fixture uses `b />`.
Existing recovered-input refusals are unchanged. This is not a whole current
dependency rebuild, whole-product Actions, performance or merge receipt.

The unchanged authoritative generators record actual Glyph source consumption.
Fresh exact-head Actions, full instruction gates and protected queue validation
decide publication acceptance. Existing formatter entry points, default routes,
options, legacy fixtures and dependency manifests are unchanged.

Remaining work includes dynamic/full directive forms, genuine retained embed
documents, every Vue dialect, SFC assembly, public formatter options and checked
span edits. The complete formatter and [#6882](https://github.com/ubugeeei-prod/vize/issues/6882)
fix-history gate remain open; this slice does not replace a product route.
