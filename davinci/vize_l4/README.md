# vize_l4

`vize_l4` is Davinci's L4 emission level. Every output target writes text
directly into one append-only `write::Writer`: indentation, the used runtime
helpers, and span links that compile away when the writer does not record.
Preambles are joined after the body, so nothing is inserted mid-text.

`write::EmitDocument` and `write::SpanLink` are the finished document and its
generated-to-authored links; `write::source_map` serializes them as Source
Map v3. The production compiler keeps its own document until its fix-history
fixture gate closes ([#6880](https://github.com/ubugeeei-prod/vize/issues/6880)).

`module::assemble` joins prepared hoist/cache, script and render writers into
one document. It supports script-only components, declared client/server render
functions and inline setup render expressions. Its late import preamble uses
the supplied helper vocabulary and preserves all fragment links. The script
lane owns default-export rewriting and inline insertion points; the assembler
does not parse JavaScript or admit a target.

`runtime::vocabulary` owns compiler helper names for the workspace's selected
Vue 3.5.35 DOM/SSR release and Vue 3.6.0-rc.9 Vapor release. Checked helper IDs
are local to that exact vocabulary; `vocabulary_for` rejects other version
pairs. `ModuleParts::for_runtime` selects this provider before assembly. SSR
imports its helper group from `@vue/server-renderer` and shared helpers from
`vue`, preserving first-use order within each group. Custom vocabularies may
also describe multiple module groups.

Expression rewriting (`expr`) and the DOM, SSR, Vapor and type-check targets
(`targets`) remain unfinished in [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).
Older Vue runtimes, complete generated-script helper admission and product
integration still need their actual providers and compiler fix-history gate.

The crate remains unpublished while its targets and consumer migration are
unfinished.

## License

MIT
