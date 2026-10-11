# Empty source range intersections

Tracking issue: [#8639](https://github.com/ubugeeei-prod/vize/issues/8639).

`SourceRange` describes half-open byte ranges. Its public constructor preserves
the supplied endpoints, and `is_empty()` and `len()` already treat equal or
reversed endpoints as empty. Keep that contract: an empty range never shares a
byte with another range and therefore never intersects it.

The old predicate reports an intersection for `15..15` or `16..14` against
`10..20`, even though `intersection()` returns `None`. Reject either empty
operand before applying the existing overlap comparison. Keep the function
`const`, the constructors, wire fields, valid-range behavior and return types
unchanged. Carton continues to re-export the identical L0 type.

The corpus fixes sixteen ordinary, point, reversed, adjacent and `u32::MAX`
boundary examples. The L0 integration test consumes those exact fixtures in
both operand orders and checks agreement with `intersection()`. A second law
compares all 2,401 pairs with endpoints in `0..=6` against actual membership in
standard half-open Rust ranges. A third covers ranges collapsed by saturating
offsets and preserves their existing length and containment behavior.

The unchanged source-range implementation compiled with Rust 1.99.0 in an
isolated wrapper; the same finite-membership and empty-range laws both failed.
The wrapper retains the real method bodies and removes only the unavailable
Serde derive, so this is a narrow before-fix reproduction, not repository
qualification. Fresh exact-head Actions and actual protected merge remain
required before delivery.

First source Check `38142399359` rejected the numeric-range `contains()` call
inside an assertion under the existing conservative partial-output lint. Keep
the same complete Boolean observation in a named local before its assertion.
Do not change the lint or its allowlist; all fixture bytes and production
behavior remain exact. The successor requires fresh complete Actions.

This independent foundation leaf adds no provider, pipeline stage, product
route or host dependency. It changes no benchmark input or numeric ceiling.
Whole #6834, #6826 and #6100 remain open; host isolation, native product parity
and released consumer acceptance retain their own gates.
