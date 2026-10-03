# Original selected-template output custody

Issue: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The neutral `Writer`, `Emitted` and `EmitDocument` APIs deliberately permit
prepared fragments and caller-selected source strings. Those APIs cannot
establish original selected-template ownership for a complete native output.

`emit_template_output` instead consumes the genuine non-cloneable
`NativeTemplateDomAnalysis`. Its private constructor derives `SourceBlock`
only from that analysis's retained selected Component, checks the original
whole-source pointer against the same File, and uses the existing DOM encoder
and standalone assembler with the existing Vue DOM vocabulary. No additional
AST walk, semantic decision pass, source parse or pipeline stage is introduced.

The sealed output normally owns the moved analysis, its original source frame,
the complete recorded document and the actual helper set. It exposes read-only
code, links, helpers and analysis. `source_map(filename)` derives its complete
`sourcesContent` and authored coordinates from the retained original root.
The filename is presentation metadata; there is no external source argument,
mutable document accessor, fragment extraction or caller-writer constructor.
The copied source-frame getter carries metadata, not completion authority.

Before sealing, every generated range must lie on real UTF-8 boundaries in
the complete module, and every authored range must lie on real boundaries
inside the original selected block. Legitimate empty modules keep empty
links and mappings. Typed emitter/assembler/range refusals retain the same
analysis and normally owned original File/input diagnostics without publishing
partial native output.

The original pipeline laws cover a Unicode original comment before the
template, five JavaScript line terminators, non-BMP text/attribute/literal
bytes, independently decoded whole-module VLQ positions, complete source-map
envelopes, separately allocated byte-equal original owners, valid empty maps
and retained `search`/legacy-literal/RegExp refusals. Compile-fail laws reject
neutral writer/document/frame promotion, private construction, cloning,
foreign-source substitution, analysis reuse and invalid owner/output borrows.

The existing twelve complete literal captures now obtain their actual module,
map and links from this sealed output and keep all frozen bytes/hashes and
pinned Vue 3.5.35 execution laws unchanged. The existing once-only protected
first-tooling-shard action additionally executes the new custody target and
compile-fail docs. Source and protected execution must both pass on the actual
head; historical captures grant no new merge credit.

This is a standalone render-export custody slice. Whole-SFC script/component
assembly, product/default routing, unsupported source families, full upstream
map envelopes and Text/Comment/Attribute subspan parity remain unfinished.
The neutral assembler is unchanged; independent whole-SFC product owners must
derive their own assembly from their genuine complete source observations.
