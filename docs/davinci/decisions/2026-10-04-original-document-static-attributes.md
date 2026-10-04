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
Admitted start tags contain at most 64 authored attributes (duplicates count)
and 16 KiB of original opening bytes; explicit typed refusals retain larger
sources. These bounds constrain the native provider's duplicate/name readback,
not original lexical input size or total work across repeated consumer queries.

These rules follow the [HTML tokenizer attribute states](https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state)
and [input preprocessing](https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream).
Ten complete original JSON corpus sources bind native effective name/value
arrays to independent actual Chromium DOMParser, including empty/Boolean quotes,
unquoted slash, duplicate/NUL/Unicode names, metadata/void/envelope attributes,
literal control characters, entity boundaries and once-only multiscalar values.
Eight runtime laws also retain exact source/quote geometry, owner/arena movement,
independent equal-buffer ownership and original lexical/Vue refusals. Two
compile-fail laws retain private construction and non-Clone authority.
The unchanged storage scanner records only the new test-owned Vec/String
bindings (1/5 and 1/15 direct/bound counts); attribute production owns no buffers.
Actual iteration laws traverse all 64 long common-prefix names and mixed
multiscalar/numeric-CR/literal-CRLF/once-only values, plus an exact 16 KiB opening
value. Both 65 unique/duplicate attributes and one-byte-over openings refuse.
These are execution laws, not a new workload instruction benchmark or speedup.

The first hosted source fails unchanged Clippy's indexing/slicing gate at two
borrowed event-range reads. Checked get-based readback corrects both sites
without relaxing the lint, changing original inputs or granting runtime credit.
The next source passes production Clippy but its new boundary test compilation
finds no_std trait omissions in test-only string generation. L0 CompactString
construction and the actual ToCompactString trait preserve all generated source
bytes, bounds and assertions without adding an std prelude or production change.
The successful 0d453569 hosted100+4 proof is historical after this source edit;
fresh exact-head runtime/browser and instruction proof remains required.

## Unfinished gates

The existing explicit no-quirks envelope, twelve ancestry sources and refusal
boundaries stay intact. DOM text/comments, implicit ends, tables, foreign/raw
content, general HTML policies, petite-vue scope/effect/L2 and all five product
history/default/zero-fallback gates remain unfinished. URL resolution and
Boolean-property semantics are outside static attribute value readback.

Fresh exact-source Actions must execute the native laws and actual Chromium
lane. Protected full suites, unchanged instruction ceilings/ratchets and actual
signed merge/fresh-main source identity still determine delivery; source checks
or queue entry alone give no completion credit. This slice additionally requires
genuine hosted unchanged 100+4 instruction measurement before queue entry;
protected measurements still repeat on the actual composed candidate. #6836
remains closed.
