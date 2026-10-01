# L1 authored binding ownership

This private provider slice follows
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
and the [retained expression ownership](./2026-10-01-l1-retained-expression-handoff.md)
contract. It prepares genuine bindings for a future dialect-built ForHead; the
[ForHead proposal](./2026-10-01-l1-for-head-proposal.md) remains unimplemented.

`NativeSyntax::into_slot_params` consumes the existing SlotParams artifact and
returns `RetainedSlotParams`. Original FormalParameter roots retain their
existing arena slice and all descendant addresses. A rest root moves out of its
generated parameter container into the shared arena; its original binding
descendants remain unchanged. The generated arrow, empty body and parameter-list
container stay private and never acquire authored source ranges.

The returned parameter/rest references have the allocator lifetime and can
outlive the observation owner. Comments retain their original arena slice;
complete Diagnostics remain normally owned and drop with the observations.
Syntax, admission and module-context holes preserve source and observations but
expose no recovered bindings. Wrong-shape rejection returns the intact artifact
in an ordinary owned box, following the expression handoff's cold-error policy.
No parser call, clone, AST walk, unsafe operation or leak is added.

Coordinates carry the real generated prefix and already checked source/decode
map. Consumers must use exact decoded/authored projections. Binding patterns
and defaults remain actual OXC syntax, never expression approximations or text
names. JS/TS admission and all existing parse-unit/context limits still apply.

Five native laws cover original parameter/descendant/comment-slice pointers,
roots after owner drop, JS/TS annotations, once-decoded entity maps, complete
failure observations, empty/rest-only lists, generated-range rejection and
intact wrong shapes. All 51 inherited/new laws and strict Clippy pass in the
scoped actual-source harness with pinned OXC. This is module evidence; exact-head
Actions, full suites and protected queue delivery remain pending.

ForHead range partition, sparse binding positions, L2 scope resolution and
structural lowering remain unfinished. This handoff does not parse a sparse
ForHead list as a SlotParams list or invent placeholders for absent aliases.
