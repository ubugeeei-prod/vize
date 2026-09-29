# SFC slot-scope option compatibility (2026-09-29)

Issue: [#6898](https://github.com/ubugeeei-prod/vize/issues/6898). Release: [v0.429.2 PR](https://github.com/ubugeeei-prod/vize/pull/6977).

The scoped-slot correction in #7142 added `no_slotted` to two public Rust
structs: `vize_l1_to_l2::DomEmitOptions` and
`vize_relief::CodegenExperimentalOptions`. A downstream full struct literal
from v0.429.1 then fails to compile, even if the caller never uses scoped
SFC styles. The v0.429.2 patch restores those original field sets.

The SFC still computes `has_scoped && !slotted` once from its own style
blocks. It passes that value through additive SFC compile and codegen entry
points, then into each emitter's invocation-local context. Existing public
entry points pass `false`, which is their previous direct-template default.
No process-global or thread-local mode carries this value. L2 fast emission,
observed/captured emission, and the compatibility codegen path receive the
same SFC value. The slot outlet still emits Vue's fifth `renderSlot`
argument when needed; scoped styles with `:slotted()` retain their ordinary
call. The old option literals remain exactly constructible.

Source verification: external-style full literals compile in
`public_options_literal.rs` and `public_experimental_options_literal.rs`.
`scoped_slot_no_slotted.rs` pins fast and compatibility render bytes and
checks three simultaneous SFC compiles with distinct slot policies.
The protected queue's full instruction-count and output gates remain the
release acceptance gate.
