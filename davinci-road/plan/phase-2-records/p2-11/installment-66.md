# P2-11 Installment 66 - Template-Wrapper Component Props

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5533](https://github.com/ubugeeei-prod/vize/pull/5533), merged
> 2026-08-31 at `d7040e03d`.

This installment keeps component props emitted through template wrappers on
the shipped DOM lane's hoist and inline-props surface. Static component props
inside scoped slots and template `v-for` wrappers can hoist, but scoped-slot
dynamic-param props stay inline and branch keys are not duplicated beside an
authored key.

The durable witnesses are:

- [`davinci_s2_template_wrapper_component_props.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_template_wrapper_component_props.rs)
  - compares reduced template-wrapper component-prop cases byte-for-byte
    against the shipped DOM lane.
- [`component.rs`](../../../../crates/vize_s1_to_s2/src/emit/component.rs)
  - owns the component static-props hoist decision at wrapper boundaries.
- [`props_static.rs`](../../../../crates/vize_s1_to_s2/src/emit/props_static.rs)
  - records whether component props contain dynamic values, non-key props and
    valued props before hoist selection.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
