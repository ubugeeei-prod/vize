# Plain scripts retain existing native diagnostic ownership

Decision for [#7943](https://github.com/ubugeeei-prod/vize/issues/7943).

Corsa already collects authored js/mjs/cjs/ts/mts/cts diagnostics. The sync
collector first parses these documents as SFCs, so a generic type argument
creates a blocking malformed-block diagnostic before the native path runs.
Share the existing physical-extension classifier with sync and lint-only
collection and initial virtual-document construction. Plain scripts avoid the
SFC route; native diagnostics retain their original source/options/revision.
JSX/TSX, standalone HTML, Art and actual Vue routes remain their existing owners.
The client languageId does not override the physical source dialect.

Retain all three complete original Map files and the entire reporter client.
Real source-built stdio pins complete original no-config diagnostic publications
for every reported extension/languageId pair, including the misleading JSX id.
Separate mandatory-native sessions retain complete TS2322 diagnostics through
actual versioned unsaved errors/repairs/restoration, generic type guards,
TS/MTS/CTS, JS/MJS/CJS, mismatched Vue/JSX ids, LF/CRLF and astral UTF-16 ranges.
The same complete Map input on a Vue URI retains its full malformed-block
publication, proving SFC parser errors remain owned by the real SFC surface.

This is a narrow current-product routing correction, without a new parser,
backend, source sweep, pipeline stage or legacy-backed native feature credit.
No instant or 10x gain is claimed. Fresh exact-source Actions, all unchanged
protected instruction ceilings, actual merge/release and installed RPC proof
remain required.
