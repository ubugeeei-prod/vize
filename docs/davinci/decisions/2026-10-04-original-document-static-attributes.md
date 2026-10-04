# Original Document static attribute receipts (2026-10-04)

Issues: [#6835](https://github.com/ubugeeei-prod/vize/issues/6835) and
[#6843](https://github.com/ubugeeei-prod/vize/issues/6843), both unfinished.

## Decision

Expose sealed immutable static attribute receipts on genuine bounded
`DocumentHtmlElement` views. The existing native insertion loop records only
its actual start-tag callback range on the existing arena-backed Element.
Readback borrows the sole original Document event tape and owner; caller-built
tokens, Component views or equal-input independent parses cannot mint it.
No parser, entity decode, pipeline stage, serializer, per-attribute persistent
row, name set or decoded string buffer is added.

Authored name/content spans and original NoValue/Single/Double/Unquoted quotes
remain distinct. Empty quoted values retain an empty original content span;
Boolean attributes retain no value span and yield the empty HTML value.
Unquoted slash remains part of the value. The shared lexer already owns
attribute-context ambiguous ampersands and actual complete entity payloads.
Scalar value readback consumes those retained payloads once, including numeric
Unicode corrections and multiscalar named expansions. Literal CRLF/CR becomes
LF and literal NUL becomes replacement; entity-origin CR bypasses input
preprocessing. Neither reference boundaries nor once-decoded literal ampersands
are normalized or decoded again.

Name scalars fold ASCII uppercase and replace authored NUL, preserving
non-ASCII case. Effective attributes retain the first matching normalized name
in original order. The iterator compares borrowed earlier original name events;
this trades quadratic duplicate filtering within a tag for zero name-set
allocation. It is not a measured performance improvement. The callback range
adds two usize fields per existing bounded Element (16 bytes on 64-bit targets),
without altering published Component/tokenizer storage or live product paths.

These rules follow the [HTML tokenizer attribute states](https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state)
and [input preprocessing](https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream).
Ten complete original JSON corpus sources bind native effective name/value
arrays to independent actual Chromium DOMParser, including empty/Boolean quotes,
unquoted slash, duplicate/NUL/Unicode names, metadata/void/envelope attributes,
literal control characters, entity boundaries and once-only multiscalar values.
Six runtime laws also retain exact source/quote geometry, owner/arena movement,
independent equal-buffer ownership and original lexical/Vue refusals. Two
compile-fail laws retain private construction and non-Clone authority.
The unchanged storage scanner records only the new test-owned Vec/String
bindings (1/4 and 1/8 direct/bound counts); attribute production owns no buffers.

## Unfinished gates

The existing explicit no-quirks envelope, twelve ancestry sources and refusal
boundaries stay intact. DOM text/comments, implicit ends, tables, foreign/raw
content, general HTML policies, petite-vue scope/effect/L2 and all five product
history/default/zero-fallback gates remain unfinished. URL resolution and
Boolean-property semantics are outside static attribute value readback.

Fresh exact-source Actions must execute the native laws and actual Chromium
lane. Protected full suites, unchanged instruction ceilings/ratchets and actual
signed merge/fresh-main source identity still determine delivery; source checks
or queue entry alone give no completion credit. #6836 remains closed.
