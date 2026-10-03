# Original Vue style custody (#6837, #6836, #6840)

The source-owned Vue descriptor now classifies inline `<style>` blocks at the
original splitter's existing emission event. Private arena selections record
their actual container indices and checked whole-root `SourceBlock` content.
`AdmittedDescriptor.styles()` lends read-only `StyleView` values in authored
order. Each view borrows the original owner, original block and ordered raw
attributes; it cannot be made from a public capture or substituted source.

Styles are opaque source data. No CSS parse, entity decoding, language inference,
preprocessing, scoped transform, CSS module resolution or `v-bind()` analysis
runs here. Absent `lang`, raw nonempty languages such as `css`, `scss` and
`less`, and raw `scoped`/`module` attributes are structurally retained. Bare,
empty and named module values keep their original distinction. These attributes
grant no stylesheet semantics, and never select script/template language.

The bounded Vue 3 descriptor policy still refuses external `src`, encoded/empty
language values, duplicate/ambiguous/unknown attributes, noncanonical block
spelling and self-closing/missing/uncertain boundaries. Custom blocks remain
retained refusals. The full original capture survives every refusal, including
all source bytes, original attributes and unchanged splitter diagnostics. A
style alone never satisfies the template/script component-cardinality rule.

Script and template parsing use their genuine original source owners when this
structural style family is admitted. The original splitter and its complete
capture contract are unchanged. There is no additional source scan, pipeline
stage, serialization, legacy parser or normal/build dependency. No files are
renamed, so a move-only commit or rename script is not applicable.

The native compiler's output currently contains only a JS module and source map.
Its existing success contract cannot silently omit style blocks. The same change
therefore requires `StyleCompilationUnavailable { container_index, span }` before
module emission for every admitted style. Original descriptor/script/template
and File observations remain available beside that product refusal. External
style sources keep the existing `ExternalBlock` refusal and are never loaded.

Meaningful laws cover multiple styles interleaved with real script/template
roles, Unicode/CRLF/raw entities, source/block/attribute pointer identity,
unchanged complete capture parity, empty style content and raw source profiles.
Refusal laws retain original data and diagnostics for external, duplicate,
ambiguous, unknown, language and boundary input. Orchestration laws retain real
script syntax and template/File owners; product laws prohibit successful modules
that discard styles. The reviewed storage inventory adds only the two arena
style-selection owners and their existing constructor use.

TODO: a native CSS source provider must establish real stylesheet and `v-bind()`
facts before a compiler consumer can return CSS and inject matching runtime
bindings. The existing full SFC result has combined CSS; even unscoped ordinary
styles may require `v-bind()` transformation, trimming and authored-order joins.
Raw passthrough alone does not establish that output contract. Scoped/module
semantics, preprocessors, custom/external-block resolution and full dialect
coverage remain separate work. Whole #6837/#6836/#6840 and all product fixture
gates stay open. Exact-head Actions, unchanged instruction ceilings and the
protected queue's actual merge are required before this slice is accepted.
