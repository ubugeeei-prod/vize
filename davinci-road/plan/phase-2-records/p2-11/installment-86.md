# P2-11 Installment 86 - Binding Metadata

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5620](https://github.com/ubugeeei-prod/vize/pull/5620), merged into
> the installment-84 branch and carried to `origin/main` by
> [#5615](https://github.com/ubugeeei-prod/vize/pull/5615) at `e9923c0f8`.

`binding_metadata` in its non-inline shape - the dev-server shape of a
`<script setup>` component. `DomEmitOptions::bindings` holds a neutral
`BindingTable` / `BindingKind` pair rather than the shipped types, so the
stage library keeps its own vocabulary.

Prefixes come from `BindingType::non_inline_template_prefix`
(`$setup.` / `$props.` / `$data.` / `$options.` / `_ctx.`); destructured prop
aliases project through `rewrite_props_aliases`; a component tag resolves
exact, camelized or PascalCased to `$setup.Name`, with props only as a
fallback and a dotted suffix kept; a `vFoo` directive reads
`$setup["vFoo"]`; an Options API handler is the guarded `(...args) =>`
reference; and module mode takes the six-argument render signature.

**The shipped lane is asymmetric here**, and the port has to be too: the
transform skips `resolveComponent` on a *verbatim* tag match while the codegen
skips the asset through the camelize / Pascal widening. The helper-preference
walk therefore matches the verbatim form, or the import ordering diverges -
seven corpus files said so.
