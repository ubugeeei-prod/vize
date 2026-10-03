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

The checked expression writer consumes actual retained L2 resolution facts.
`targets::dom::emit_file` consumes only the genuine file-owned L3 analysis and
emits static DOM structure and retained literals. Every runtime reference is a
typed refusal until the real same-file Vue access provider exists. Complete
template modules are assembled with `module::assemble_template`; authored maps
retain the whole file and original retained expression coordinates.

`targets::ts::project_program` borrows one completed File with one genuine
whole-source JS/TS Module unit and no template operations. It copies that
Program through the existing writer, keeps its original checker language and
exposes checked authored diagnostic ranges. Generated module punctuation is
unlinked. This additive input does not select Canon's default route or admit
Vue templates, JSX, incomplete File syntax or the full type-check target.

Whole-component completeness, file If/For, broader DOM semantics and the SSR,
Vapor and type-check targets remain unfinished in
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).
Older Vue runtimes, complete generated-script helper admission and product
integration still need their actual providers and compiler fix-history gate.

**Experimental.** The crate follows the
[checked Rust support contract](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).
It is registry-eligible for the explicit native SFC dependency closure. Actual
first publication, Trusted Publishing handoff and release acceptance remain
unfinished, together with the targets and default consumer migration above.

## License

MIT
