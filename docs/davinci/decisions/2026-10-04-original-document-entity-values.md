# Original Document character-reference values

This independently reviewable Stage 1 slice advances [#6835](https://github.com/ubugeeei-prod/vize/issues/6835)
and the Document substrate needed by [#6843](https://github.com/ubugeeei-prod/vize/issues/6843).
The issue comment and central markup paragraph record the same decision.

The existing `NativeDocument` recorder discarded the native lexer's actual
`DecodedEntity` payload while retaining its authored span. It now stores text
and attribute entity values as private event variants. Numeric references keep
their corrected Unicode scalar; named references borrow the existing static
table expansion, including two-scalar references. The original immutable
`DocumentToken` exposes that value beside its genuine owner and authored slice.
One reference remains one original callback event. Reading it neither decodes
again nor constructs a replacement buffer, AST, pass or serialization stage.

The same existing bounded explicit-envelope HTML construction classifies those
actual retained values. Outside its body interval, only TAB, LF, FF, CR and SPACE
expansions preserve admission. Unicode spaces, mixed/nonspace expansions and
once-decoded ampersand literals retain the original explicit-envelope refusal.
This follows the character-token rules in the [HTML insertion modes](https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode).
Ordinary body entity observations retain their prior element-only admission.

The private event representation stores the entity payload in the same arena
event allocation; it adds no separate entity table or per-reference heap
allocation. That event type is larger because genuine original values now
survive readback. Existing Component and published tokenizer storage, callbacks,
product routes, fixture expectations and all instruction ceilings stay intact.

Seven new native laws cover one multiscalar callback, numeric Unicode correction,
attribute/text ambiguity, single decoding, nonentity observations, owner moves
and preservation of recovered syntax. Eight explicit nonspace inputs retain
typed structure refusals. The unchanged eight shared HTML ancestry sources are
preserved byte-for-byte; four new original sources exercise entity whitespace
before/between envelope members, head metadata and once-only body values.
Rust compares all twelve native ancestries, and the existing Chromium suite
checks those same complete sources, HTML namespace, parent order and no-quirks
mode. These are authored laws, not proof that a whole browser profile is done.

The original static-attribute value/assembly, decoded text-node construction,
implicit envelope/end, table, foreign/raw/formatting and Vue-flavored refusals
remain. Petite-vue scope/effect meaning, L2 and product admission, fix-history
closure and default migration stay unfinished. #6835 and #6843 remain open.

Pure Rust formatting, storage inventory and dialect-isolation checks precede
publication. Exact-source GitHub Actions, actual browser execution, full
protected-queue checks with unchanged instruction ceilings and actual merge
are still required for delivery; none is inferred from source inspection.
