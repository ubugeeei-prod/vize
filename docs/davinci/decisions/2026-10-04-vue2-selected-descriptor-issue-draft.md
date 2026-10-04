# Prepared issue pairing: Vue 2 selected descriptor owner

Status: local review draft only; not posted. Intended pairing: #6837 and #6842.

Proposed comment body for both issues:

> The next lowest-stage historical gap is original selected-descriptor
> ownership. Actual main `6987c523ecd2bee6e8489e2973e4b7499ce25018` has genuine
> Vue 2 Component/TextView and borrowed expression providers, but the shared
> Descriptor entry correctly refuses every non-V3 profile.
>
> Privately implement a separate sealed `Vue2DescriptorObservation` and borrowed
> `Vue2TemplateView`: explicit V2/Vue/default options, one normal template,
> scriptless/styleless, default delimiters, retaining the same original
> splitter root/block/index/opening/closing spans and normally owned
> `vue2::surface::ComponentParse`. Run the existing splitter once; after all
> envelope checks call the existing `parse_component_block` once. Reuse
> original template classification at its callback without making the V3
> public Descriptor entry accept Vue 2. Every script/style/custom/external,
> wrong-profile, duplicate or unsupported boundary/attribute remains refused.
>
> Envelope custody does not certify clean body grammar: original Component
> errors, recovery, filter/base/argument syntax and TextView refusals remain
> retained and independently checked. No raw capture, modern selected owner,
> caller source/AST tuple, extra parser/walk, legacy bridge or File/runtime
> admission is introduced. The Glyph TextView-to-Doc lane remains independent.
> HTML comments supply genuine positive CST controls; expression comments
> retain the actual `CommentSyntax` refusal, never a fabricated admitted view.
>
> Required source/lifetime/foreign/sibling/movement/drop/unwind/count laws,
> original full-source/CST geometry, existing V3/Vue 2 oracles, honest storage
> inventory, exact-head Actions and unchanged protected 104 benchmarks precede
> later actual merge. Whole-SFC File, legalization, registry/classic runtime,
> code/maps, product history/default routes and wider dialects remain open.
>
> Root authorized private implementation after corrected design e683 and
> both retained source-only peer clearances. Twelve unit laws, one existing
> isolated heap-drop harness case, ten correctly typed compile-fail controls
> and one positive doctest are authored, with real per-thread splitter/parser
> entry and after-parking fault hooks only under cfg(test). Component and
> NativeSyntax diagnostics remain normally owned. This is unexecuted source:
> no Rust build, Actions, publication, manual campaign or queue admission has
> occurred. Root and retained peer review of the frozen source is required
> before those later acceptance steps.
>
> Companion: `docs/davinci/decisions/2026-10-04-vue2-selected-descriptor-plan.md`;
> the central level-restructure record carries the same bounded plan.
