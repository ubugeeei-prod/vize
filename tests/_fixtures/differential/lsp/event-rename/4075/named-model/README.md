# Named model editor transactions (#4075)

This separate test-only corpus owns 12 contexts: child declaration, `v-model`
argument or `update:` listener origin, with LF/CRLF and `nextValue`/`next-value`
replacement spelling. Each context retains both camel and kebab parent usages.
The 224 authored UTF-16 cursor positions include every character of the selected
origin sites. One parent site follows an astral glyph on the same line.

The complete input and updated files are authored independently. The source-only
`author_contracts.py` enumerates literal coordinates and expected packets; it
does not run or inspect the product. Frozen `cases.json.txt` has SHA-256
`dce60b59a09de699b8ba9aa928463790ff14627c8d5f6d46c8a50e23886c70b8`.

Before querying the native editor, the test binds only the three exact temporary
file URIs into the expected packet. Request IDs, source ranges, complete reply
envelopes and array ordering remain contracted. Every cursor has a prepare,
reference, definition and rename contract: 896 whole response checks. Prepare
accepts the two explicit standard range representations; definition accepts a
single Location or its one-element array. Neither permits a different range,
generated target, extra response field, URI alias or filtered subset.

Every context requires the real workspace native runtime and frozen Playground
Vue dependency. Its independent TS2322 witness and repair must publish complete
matching diagnostics. All three initial documents must be clean. The first
query is the fixed transaction for that origin; the test applies its complete
returned edits to memory and disk, checks all version-2 files and diagnostics,
then independently installs the full authored goldens and checks version 3.
Other's same-name model declaration and both usages, local strings and handlers
must remain unchanged. Foreign/resource/overlapping edits refuse as a whole;
missing, null or cross-component edits cannot match the full goldens.

The separate `named-model-transactions` capture namespace keeps each complete
case, expected packet, actual replies, disk text and all diagnostic publications.
The existing process helper retains its original decoded RPC/terminal evidence
and passive protocol. No capture flag or runtime deadline changes.

This preparation has not executed against its source producer. The existing
basic model test, bound-emitter corpus and stock Content Mapper null-rename
controls remain unchanged. This corpus grants no current-source, protected,
installed-release, upstream-host or umbrella #4075/#6258 completion credit.
