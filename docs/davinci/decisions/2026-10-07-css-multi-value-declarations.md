# CSS multi-value declaration layout (#7968)

Issue: [#7968](https://github.com/ubugeeei-prod/vize/issues/7968).

The existing CSS print joins authored continuation values. Restore complete
one-value-per-line layout for transition and box-shadow lists, including compact
inputs, and preserve an authored multiline list for other ordinary properties.
Vendor-prefixed transition/shadow declarations follow the same rule. Single
values, function arguments, quoted commas and custom-property token streams
retain their existing ownership. The existing nested-comment policy stays in
place. Other compact property-list wrapping policies remain outside this slice;
this implements both reported properties and the issue's minimum preservation
requirement for other authored lists.

The source uses the existing whole-token alignment pass after the existing
LightningCSS parse/print and authored-value selection. Pending continuation gap
edits commit only at a declaration boundary in a block owned by the existing
parse; nested selectors and custom brace contents cannot establish ownership.
A complete token mismatch discards every pending edit. One continuation indent
uses configured tabs/width/EOL. No CSS parse, pipeline stage or serialization is
added. Parsed transition/shadow list lengths prevent the equal-layout fast path
from skipping those lists, while unrelated equal CSS retains that fast path.

The additive, independently authored corpus contains 26 full SFC references:
original Tip/Card, compact and already-printed lists, hybrid shadow indentation,
three values/important, vendor prefixes, nested authored shorthands, selector
lists, scoped/module, comments, function/quoted/custom commas, ordinary authored background lists, tabs, width and CRLF/Auto. Rust
checks original/reference CSS parse/print semantics and three complete actual
public-API passes. The existing source observer reuses its build and records raw
options/API observations plus 21 default CLI check/dry/write/recheck controls.
No historical input/output/capture asset is changed.

Validation and delivery are pending: exact-head Actions, actual protected
historical corpus/API/CLI qualification, all unchanged instruction ceilings,
actual merge, and supported installed-CLI release proof. No speed, native
formatter migration or fix-history completion is claimed. Root owns queue
admission and release so independent slices can progress during hosted checks.
The reporter's verified public credit is
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
