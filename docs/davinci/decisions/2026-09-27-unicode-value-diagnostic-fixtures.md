# Value-sensitive Unicode diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Historical fix: `13e5ec2a9752c5d8d41a9510f54e7096a583bb78`, which has no
historical issue number. The fixture metadata records that number as null.

## Decision and scope

Turn the preserved non-ASCII expression requirement into an actual diagnostic
contract. Two SFCs declare the reserved prop bindings `default` and `static`;
their key/ref expressions call functions accepting exactly the literal types
`"日本語"` and `"😊"`. The prop accesses force the production reserved-binding
rewriter to operate on these expressions.

The valid project requires no diagnostics. The invalid project passes the
literal `"wrong"` to both functions and requires exactly two actual TS2345
errors in App.vue, line 7, UTF16 columns 31 and 78. Each complete message names
its exact Japanese or emoji parameter type. The test compares the whole batch
list, with all available public fields and returned order.

Both projects are required T1 fixtures. Their ordinary test body always
executes the unchanged production checker; exact-name T0 nextest deferral must
be paired with required full T1 Cargo execution. The existing runtime fixture
packs and their numeric historical issue metadata remain compatible with the
nullable historical-issue field.

## Verification and remaining work

Actual TypeScript 7.0.2 / real Vue 3.6.0-beta.10 runs observed zero valid errors
and the two exact invalid errors. The initial invalid draft deliberately failed
before its actual rows were frozen.

Fault injection replaced quote copying in the current production rewriter with
UTF8-byte-to-character decoding. The mutated library compiled, then produced
two real mojibake TS2345 errors on the valid fixture, making its equality test
fail. This models the historical corruption within the current implementation;
the original historical revision was not rebuilt. The production source was
restored byte-identically before normal verification. This control demonstrates
that the fixture detects the value corruption rather than merely containing
Unicode text.

The restored normal first/repeat runs passed all four fixture tests and eight
projects, with identical observed Unicode lists. Strict Clippy and formatting
also passed. The temporary production mutation is absent from the prepared
change.

Actions/full T1 queue proof, common differential execution, native L4 parity,
end ranges, related information and untouched backend observations remain open.
The broader historical patch's template-literal/comment paths still need
separate evidence. Native acceptance is zero and #6879 stays open.

## Exact T0 deferral

Stack this slice on the required-props draft #6940. Extend only the present
package `vize_canon`, binary `fix_history_diagnostics`, exact test
`unicode_reserved_props_preserve_value_diagnostics`. Prior Canon/Corsa names remain exact;
source existence and T1 metadata are checked. Full Cargo T1 stays unchanged.
Draft Actions and CI parent main/full queue proof remain pending.
