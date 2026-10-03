# Native plain CSS output (#6837, #6836, #6840)

The native SFC result now returns optional combined CSS alongside its existing
module and map. `css_document()` retains the emitted CSS and original full-file
span links; `css_source_map()` returns its separate map when `source_map` is
enabled. Styles remain in the complete original descriptor observation, including
empty blocks and authored attributes. No product default or shipped route changes.

Only a genuine admitted `StyleView` can create the private `PlainStyle` receipt.
It accepts absent `lang` or exact `css`, with no other style attributes. Scoped,
module and preprocessor profiles return `StyleCompilationUnavailable` while
retaining original owners. External/custom blocks keep their earlier typed
refusals and are never loaded or omitted.

The receipt scans original content for a conservative absence proof of Vue
binding syntax. Any raw case-insensitive `v-bind` spelling, including inside
strings/comments, comment-bridged spelling, backslash escape or unfinished block
comment returns `StyleBindSyntaxUnproven` with the original container index and
file-absolute evidence span. This deliberately refuses some valid CSS. It does
not certify CSS syntax, preprocessors or completed stylesheet semantics. There
is no CSS AST reparse, normalization, entity decoding or JS binding reparse.

The source writer consumes only that private receipt. Optional `style_trim`
applies the same per-block Unicode whitespace trim as the ordinary style result;
its default is false, matching the actual `StyleCompileOptions` default. Blocks
join with a newline whenever accumulated output is nonempty, including an empty
trailing block. Entirely empty output yields `None`. Plain code and mapped code
are identical. Each emitted original line has an exact authored span; inserted
join newlines are generated structure. The CSS map uses the complete original
SFC source, not a CSS-only reconstruction or shifted private buffer.

Normal code never calls legacy style/compile helpers, creates a legacy descriptor,
loads external sources, adds a pipeline stage or crosses a reverse dependency.
The whole native observation and result retain their private construction chain.
The existing module, script and template owners determine module output; CSS
without proven Vue binding syntax adds no runtime injection.

Laws compare literal expected CSS and the dev-only actual ordinary SFC result
for empty/whitespace/multiple blocks, optional trim, newline joins, Unicode/CRLF,
comments and raw entities. The existing native module remains byte-identical to
its styleless reference, including genuine JS setup output and its original
script/template/File owners. Mapping laws verify every emitted original line against
whole-file bytes and independently decode Source Map v3 coordinates. Refusal
laws retain every original style, template and File owner with exact escaped,
raw/comment/string and comment-bridged binding evidence.

TODO: real Vue `v-bind()` facts and runtime/CSS transformation, scoped selectors,
modules, preprocessors, full CSS syntax diagnostics and every remaining native
product gate remain unfinished. Whole #6837/#6836/#6840 and fix-history gates stay
open. Exact-head Actions, unchanged instruction ceilings, actual native Stack
membership when the provider is open, protected queue and actual merge remain
separate acceptance work.
