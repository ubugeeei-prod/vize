# P2-11 Installment 63 - Nested Interactive Close Identity

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5520](https://github.com/ubugeeei-prod/vize/pull/5520), merged
> 2026-08-31 at `526f400ef`.

This installment keeps same-named ancestor close tags alive after nested
interactive-content recovery. The recovery may close the nested interactive
element, but it must not consume the surrounding authored ancestor's close.

The durable witnesses are:

- [`nested_interactive_recovery.rs`](../../../../crates/vize_armature/tests/nested_interactive_recovery.rs)
  - asserts same-named ancestors remain open until their authored end tag.
- [`surface_fidelity.rs`](../../../../crates/vize_s1/tests/surface_fidelity.rs)
  - pins the S1 close identity surface.
- [`emit_nested_interactive.rs`](../../../../crates/vize_s1_to_s2/tests/emit_nested_interactive.rs)
  - compares same-named ancestor nested anchor/button cases against the shipped
    DOM lane.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
