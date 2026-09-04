# P2-11 Installment 89 - Inline Setup Bindings

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5625](https://github.com/ubugeeei-prod/vize/pull/5625), merged
> 2026-09-03 at `bd9c5d1fb`.

`inline`: the `<script setup>` read shape. The render function is inlined into
`setup()`, so the template reads its bindings straight from the closure -
`__props.` for props, and **no prefix at all** for other setup bindings. That
absent prefix is what lets the collector reach `.value` for a `SetupRef` and
`_unref(…)` for `SetupLet` / `SetupMaybeRef`.

With it: the assignment-target paren scan, shorthand expansion
(`{ n: n.value }`, never `{ n.value }`), `_unref(Comp)` component tags, bare
`vFoo` directives, `static_cache = inline`, and the two binding-aware
patch-flag rules only an inlined render function can reach -
`is_constant_interpolation` and `is_const_handler`.

**`Helper::Unref`'s order**: the shipped lane registers it on the *transform*,
and the preamble lists transform helpers before codegen ones. The emit learns
about it mid-walk, so it marks the helper after the body into the preference
list rather than the used list.

**Deliberately not included**: `hoist_static`'s inline arm
(`is_root && inline && has_only_native_element_descendants`), measured at
1,842 of 12,062 corpus templates and left to its own installment. The corpus
lanes stay on the non-inline shape at zero divergence.
