# Original Document body child nodes

This Stage 1 provider for #6835/#6843 starts from literal main
`ac1675fef35c5637c38b1e366a91d6bdfe419885`, including actual original
Document entity and static-attribute merges #7700/#7722. Live #6837 remains
open, but no SFC/container API is consumed here. The order record permits
this independently reviewable L1 slice while #6832 remains open.

## Original source and node custody

The sole NativeDocument lexer and its private retained callbacks remain the
source authority. The existing explicit-envelope HTML insertion loop records
one original closing-callback usize per Element (eight bytes on 64-bit
targets), and the first outside-body text span on the existing structure.
Existing element/stack Vecs and static attribute callback ranges remain.
There is no node Vec, decoded string, new lexer/decoder, caller event tape,
Component carrier, serialization or additional pipeline stage.

Sealed child cursors support body, div, span and the existing true body voids.
Original element adjacency selects each direct child and its retained callback
boundary skips its subtree. Contiguous original Text/TextEntity callbacks form
one borrowed text node. The view retains original authored geometry, parent,
owner and actual callback identities, including retained DecodedEntity values.
Comment views retain their original content and complete authored frames.
Owner/structure moves before borrowing remain valid; views cannot outlive their
owner/tree, and equal byte buffers or duplicate parses cannot transfer authority.

Literal CRLF/CR streams as LF. In body text, literal NULL characters are
ignored and an entirely NULL run produces no empty text node. In comments,
literal NULL becomes replacement. Retained reference scalars bypass literal
preprocessing, preserving numeric CR and full multiscalar expansion exactly
once. Comment reference spellings remain literal bytes. These rules follow
the [HTML input preprocessing](https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream),
[in-body insertion](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody)
and [comment tokenizer](https://html.spec.whatwg.org/multipage/parsing.html#comment-state)
rules within the admitted subset.

The existing ordinary `<!--...-->` validator remains mandatory. Abrupt closes,
nested openers and alternate `--!>` framing retain typed refusal; the supported
empty and long-hyphen endings require actual whole-comment oracle agreement.
Head/html child queries explicitly refuse unsupported node-parent modes.
Characters after the authored body/html closing tags can be reinserted into
body by the browser. Body child queries explicitly refuse their original span;
existing ancestry and local div/span queries remain available. Outer comments
are outside this body's node scope.

## Work and verification

Each query revisits its own direct callbacks and child boundaries. Empty-run
classification reads literal bytes, and subsequent scalar readback reads them
again; no buffer does not mean no additional work. Work is linear in visited
callbacks/literal scalars. Total source/text size, nesting depth and repeated
consumer queries gain no new bound or measured speedup claim. Existing
64-attribute/16-KiB start-tag bounds and quadratic duplicate filtering remain.

Fifteen complete original JSON sources bind native original child-node arrays
to actual Chromium DOMParser: twelve admitted sources and three real
outside-body insertion witnesses with explicit native refusal. Existing whole
ancestry/attribute/petite source bytes and goldens remain unchanged.
Seven native laws cover whole-node census, nonzero Unicode/actual event
addresses, owner/tree movement, distinct equal-buffer authority, explicit
comment/Vue/general-policy refusals, outer-mode refusals and repeated actual
512-block/32-deep subtree workload readback (1,536 original references). Seven compile-fail controls
and one positive public example cover private constructors, non-Clone views
and owner/tree lifetimes. These are written execution requirements; no local
Rust build/test or hosted execution has yet been claimed.

The test-only storage row must match the unchanged scanner. Exact-source
Actions must run these laws and actual Chromium, plus unchanged genuine
100-level/four-formatter measurements before queue admission. Protected full
Actions and those same immutable ceilings/ratchets must pass on the actual
candidate before signed merge and fresh-main source custody verification.
Release publication holds new queue admissions until the maintainer lifts it.

General HTML tree/profile completion, implied ends, table/foreign/raw-text/
formatting/template policies, petite-vue scope/effect/L2, all remaining product
acceptance/default/zero-fallback gates and whole Vue dialect completion remain
unfinished. #6882's separately closed formatter history-fixture gate grants no
native/default formatter completion. #6835/#6843 remain open; #6836 stays closed.

The same-change decision is paired on [#6835](https://github.com/ubugeeei-prod/vize/issues/6835#issuecomment-5976474663)
and [#6843](https://github.com/ubugeeei-prod/vize/issues/6843#issuecomment-5976474940).

Independent source/law review is clear on `8ceb27281767`; the workload wording
correction changes no Rust, whole fixture bytes, assertions or storage counts.
Hosted execution and protected/actual delivery remain required.

First exact source `f65fabb0e9` [Check 37176568522](https://github.com/ubugeeei-prod/vize/actions/runs/37176568522)
fails the unchanged production Clippy `collapsible_if` gate before new native
laws or doctests execute. The sole correction uses the equivalent body-tail
let-chain condition, without a lint exemption or changing original inputs,
assertions, admission, storage or scalar rules. Its old instruction dispatch
is superseded after this source edit; fresh exact-head native/browser/100+4
proof is required before admission, then protected full/actual merge.
