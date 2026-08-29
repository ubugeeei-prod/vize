# P2-11 Installment 36 - Slot Outlet V-on Props

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5207](https://github.com/ubugeeei-prod/vize/pull/5207), merged
> 2026-08-29 at `cf7fc9a22`.
> Issue: [#5206](https://github.com/ubugeeei-prod/vize/issues/5206).

This installment realizes named and dynamic `v-on` props on `<slot>` outlets
from the S2 DOM emitter without flipping the shipped compiler lane. Slot
outlet listener props use component-style event casing because they are props
passed to `renderSlot`, not native DOM listener props.

The realized cases are:

- Static slot outlet event props, inline handlers, event/key/option modifiers,
  colon events and uppercase custom event casing.
- Dynamic slot outlet event props through `_toHandlerKey`, including event/key
  modifiers and ignored dynamic option modifiers.
- Slot outlet event props combined with static attrs, dynamic binds, dynamic
  prop keys, fallback content, `v-if`, `v-for` locals and forwarded scoped-slot
  locals.
- Slot outlet `v-bind` and object `v-on` spread combinations in the shipped
  fixed order: `v-bind` object, object `v-on`, then entry props.
- Vue 2 `.native` event sugar on slot outlets.

The durable witnesses are:

- [`emit_outlets.rs`](../../../../crates/vize_s1_to_s2/tests/emit_outlets.rs)
  - exact pins for static and dynamic slot outlet listener props, including
    the shipped duplicate-key shape for duplicate listeners.
- [`davinci_s2_outlets.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_outlets.rs)
  - S2-vs-shipped byte fixtures across slot outlet event props, spread order,
    fallback, structural, scoped and Vue 2 cases.
- [`davinci_s2_dynamic_von_keys.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_dynamic_von_keys.rs)
  - dynamic slot outlet events move from `SlotOutletPropKind` refusal to the
    byte-for-byte dynamic `v-on` battery, while malformed slot event names and
    handlers keep typed refusals.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  - the source-level `SlotOutletPropKind` witness retires from the committed
    refusal census.
- [`emit_unsupported_catalogue.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_catalogue.rs)
  - `SlotOutletPropKind` remains accounted for as a guard-only bucket for
    impossible model-product pieces on slot outlets.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
