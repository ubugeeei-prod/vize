# P2-11 Installment 65 - Dynamic Component Directive Patch Flags

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5536](https://github.com/ubugeeei-prod/vize/pull/5536), merged
> 2026-08-31 at `7a98d785b`.

This installment keeps dynamic components that are only dynamic because of
`:is` plus runtime directives on the shipped DOM lane's patch-flag surface.
Custom directives still force `NEED_PATCH`, including the static-props case
where the component call would otherwise look patchless.

The durable witnesses are:

- [`davinci_s2_dirs.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_dirs.rs)
  - compares reduced dynamic-component directive cases byte-for-byte against
    the shipped DOM lane.
- [`emit_dirs.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dirs.rs)
  - pins the S2 emit output for custom directives on dynamic components.
- [`component.rs`](../../../../crates/vize_s1_to_s2/src/emit/component.rs)
  - owns the patch-flag preservation when runtime directives are present.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
