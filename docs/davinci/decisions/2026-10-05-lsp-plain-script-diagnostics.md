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
Initial plain-script cache updates retain import tracking and remove only cached
virtual documents; they do not invoke the JSX scoped-style parser. A bounded
source peer found that routing through the JSX helper would add the wrong parse.

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

First exact native execution on ecc returns the correct full TS2322 object plus
existing TS6133 unused-variable suggestions; the initial test vectors omitted
those native hints. Retain the unchanged authored inputs and pin the complete
suggestion objects for errors, repairs and JavaScript, with full guard vectors.
Every whole publication mismatch is collected across all original sessions,
then a mandatory terminal assertion rejects the test; response, version, timeout
and shutdown contracts remain exact. Native suggestions are never filtered.
Fresh current-source execution must authenticate all vectors before acceptance.

The actual protected candidate `22e1ca79` passed all three owned LSP laws
(27 sessions / 123 complete publications), but its full Check failed the existing
KeepAlive official Vapor comparison. The Node child returned non-success with
empty captured stderr, and the old helper omitted its exit status; the cause is
unknown. The preceding actual main passed that case, which does not authorize
retrying the same failed candidate or claiming a fixed cause.

The same existing PR is genuinely rebased on actual main `69b2c8c1`, retaining
every owned production, original input and complete LSP expectation byte. A
minimal test-driver observation records actual exit status (including signal),
whole stdout/stderr and input on failure. An opt-in child observation captures
existing awaited phases and errors on stderr; successful semantic stdout, all
original KeepAlive sources/vectors/comparisons, time budgets and failure gates
remain unchanged. Fresh exact-source Actions must execute that unchanged case.
If they pass, retain the older failure as unknown; the current protected complete
suite is still mandatory before actual signed merge and installed release proof.

Paired observation decision: [#7943](https://github.com/ubugeeei-prod/vize/issues/7943#issuecomment-5991040214). The driver also retains complete raw output bytes beside text, so invalid UTF-8 cannot hide a child failure.
