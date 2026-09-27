# SSR slot scope attributes follow the receiving SFC's scoped styles

Issue: [#6898](https://github.com/ubugeeei-prod/vize/issues/6898).

A slot outlet contributes `data-v-…-s` only when its receiving SFC's own
scoped style uses `:slotted()` or `::v-slotted()`. Ordinary scoped styles
retain their normal scope attributes, and unscoped slotted selectors do
not enable this suffix. Any scoped style block can enable it. Detection
matches Vue's SFC parser pattern and is computed once from the actual
style blocks; the existing unused descriptor flag is not trusted.

The SFC carries this metadata through its template options to the SSR
compiler's separate optional options carrier. An unset value preserves
Vue's direct template compiler default (`slotted: true`). Only the outlet's
own suffix is gated; an incoming slot callback `_scopeId` still forwards.
The slot callback's VNode fallback also supplies `renderSlot`'s fifth
`noSlotted` argument, preserving a real fallback function when present.
Fallback VNodes capture their owner's scope ID through Vue itself. Only
compiler-inserted scope props in that subtree are suppressed; authored
attributes remain. Both emitters restore nested state before returning.
Vue 3.5.35 exposes duplicate attributes that 3.5.43 deduplicates, so both
runtime versions are checked with complete raw output and warning arrays.

The authored corpus contains twelve layouts, the reported scoped page,
and an explicit `inheritAttrs: false` pure forwarding wrapper. Whole SSR
module expectations cover both branches. Real Vue SSR compares raw HTML
and complete warning arrays for page-component, five direct-child and
empty-slot/fallback modes. Compiler bytes are loaded unchanged against
one real Vue instance. Attribute order and fragment comments are retained.
The nested fallback retains authored attributes. A real scoped forwarder
calling `renderSlot` adds three incoming-scope observations, pinned to full
stock HTML/warnings on both versions, giving 39 observations per runtime.

Cached public 0.429.0 reproduces nine mismatches among 33 observations with
Vue 3.5.43. The complete expected programs match those 33 reference
observations. Expected programs are authored oracles, not a capture of
the changed compiler. The original forwarding wrapper's inherited
extraneous-attribute warning is retained separately in the before evidence;
the explicit pure wrapper isolates slot-scope propagation in this corpus.

Fresh source-built tests, existing raw SSR default regressions and exact-SHA
CI remain required. No Davinci native-pair credit or issue closure follows
from source preparation or expected-program validation. Publication and
release verification are tracked separately; #6898 remains open until then.
