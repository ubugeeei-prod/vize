# Sole inline interpolation width

Issue: [#7876](https://github.com/ubugeeei-prod/vize/issues/7876).

The complete second reporter case in retained comment 5987414404 is still
authentically red on actual main `3e493feb7f98ef664cdaf2ce53d1ba94cb25c86b`.
Its start tag fits the configured width, but the same physical line also owns
the interpolation and matching closing tag. Measuring the header alone joins
the whole child onto an overflowing line. The before public API test fails
the independently authored original whole expectation on pass one; its raw
receipt SHA-256 is `cb6f36ca8b3fa951b123f0f1cd7f4d89bb6092c1988dfdf6fe43b0e9fa7a84c4`.

Measure only a sole interpolation whose original source immediately adjoins
its parent opening and matching closing tags. Include the actual emitted
prefix, formatted mustache, full normalized closing tag, SFC base indent,
configured tab width and Unicode display width. When that line overflows,
break inside `{{ }}` and anchor the closing mustache at the existing parent
depth. Reuse the already formatted expression and existing literal renderer;
no extra parse, pipeline stage or serialization is added. Newly chosen breaks
honor the current suppression lock after opening the chunk. The original
multiline, composite-text, preserved-text and closing-tag emission paths keep
their existing owners.

An independent read-only review authored sixteen complete Oxfmt 0.63.0
references before reading modified production. Retain the complete original
and compact carrier, exact100/over99, closing-suffix-only overflow, spaces and
tabs4, Unicode, explicit/Auto CRLF, suppression, raw literals, owned block
comments and unchanged #7969 controls. The sealed reference artifact SHA-256
is `ebe294c117dc2e5e055575a110f68e2b5195beed5f2328f6693b3650efb27fe4`;
its provider entry and source report are separately pinned. Thirty-two stock
Vue 3.5.35 complete before/reference compiler packets execute the unmodified
template content, including its first LF/CRLF. The sealed author's declared
capture recipe is checked separately and never used for execution. All 128
before/reference states retain complete DOM HTML/text, SSR HTML and
click/submit effects, with no warnings. The unavailable MyButton
module uses an explicit deterministic slot/button/event observer; this
supplies no claim about that external module's implementation.

The public API requires whole outputs and changed flags through three passes.
The configured CLI requires eighty complete check/write/write/write/check
statuses, output streams, file buffers and unchanged configuration buffers.
The complete original #7969 and eleven continuation-prefix laws pass locally.
The current suppression-owner hash changes only after proving the original
placement law body and historical authority remain exact; all five local
history/current-reference controls pass. Every old fixture, reference,
manifest, denominator, source law and instruction ceiling remains unchanged.

Required: exact-head hosted source/runtime and whole corpus qualification,
all 104 unchanged instruction ceilings, root-owned protected queue and actual
merge, then a supported registry-installed release replay. Earlier #8252
measurements grant no composed-head performance or delivery credit. Composite
children, independent closing-tag reflow, other expression shapes and native
admission remain unfinished. Native shared accounting stays Unsupported16/16,
handled0/16 and equivalent0/16. This bounded slice does not close #7876.
