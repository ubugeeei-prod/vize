# Implicit default slot comments (#7822)

The original report uses a `Dialog`, a named `#title` template, a direct
`<!-- note -->`, and `Confirm`. Preserve that source in the compiler
differential corpus, with no native whole-product acceptance credit.

Official Vue `compiler-core` `buildSlots` at both reported 3.5.41 and locked
3.5.35 drops direct comments from implicit content beside template slots.
Without template slots, component children still retain development comments.
The exact Vapor runtime/compiler 3.6.0-rc.9 applies the same rule in
`transformComponentSlot`; its beta version does not establish rc.9 evidence.

Apply this rule to the existing static and dynamic DOM slot collectors,
the selected L2 emitter, and both SSR push/vnode collectors. Keep nested
comments, explicit default templates, and ordinary defaults. Comment-only
filler beside named templates creates no default slot. The looped-slot control
also exposed an SSR helper mismatch: register core `renderList` for the array
of slot descriptors, matching the emitted call and Vue's runtime contract.
Do not add stages, parser passes, or serialized level transfers.

Eight retained inputs exercise the original, conditional/looped/dynamic names,
comment-only filler, and three comment-preservation controls. Rust compares
every public SFC result field against the comment-free input where applicable.
Vapor SSR's existing warning remains exact, including its original descriptor
location; only that separately checked input-dependent location differs in
the metamorphic comparison. Independent official compilers then execute whole
modules on real pinned Vue: DOM mounts and cleanup, SSR push and vnode fallback
slots, and Vapor mounts and cleanup. Whole observations include slot keys and
counts where VNodes exist, tree/HTML, authored comments, and diagnostics.
DOM and SSR use `comments: true`. The public Vapor backend has no comments
option and omits authored comments everywhere; its oracle explicitly uses
`comments: false`, and all eight complete results match comment-free inputs.
This is existing Vapor behavior, not comment-preservation or native coverage.

The public `vapor: true` plus `ssr: true` request still emits
`VAPOR_SSR_FALLBACK` and uses standard SSR. Qualifying it does not implement
native Vapor SSR or close #6100/#6880. Hosted exact-head Check and protected
full suites/instruction ceilings remain mandatory before actual merge.

Sources: [Vue 3.5.35 buildSlots](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/transforms/vSlot.ts),
[Vue 3.5.41 buildSlots](https://github.com/vuejs/core/blob/v3.5.41/packages/compiler-core/src/transforms/vSlot.ts),
[Vue 3.6.0-rc.9 Vapor slots](https://github.com/vuejs/core/blob/v3.6.0-rc.9/packages/compiler-vapor/src/transforms/vSlot.ts).
