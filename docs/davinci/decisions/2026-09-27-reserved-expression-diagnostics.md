# Reserved template expression diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Historical fix: `27ae56ea668fc6c94653894ec29d75f8339a04c7` (#923).

## Decision and scope

The original fix rewrote reserved props beyond a standalone identifier. Its
unit witnesses checked generated expressions: object values, shorthand,
property keys, member accesses, strings, template literals, regexes and an
ordinary binding. Preserve those focused requirements through two new authored
SFC projects that execute the unchanged production batch checker with real Vue.
These are derived diagnostic witnesses, not byte-identical historical SFCs.

The valid SFC has typed `static`, `class` and `default` props. A direct `static`
read and a function argument object exercise reserved shorthand, a reserved
object key, `item.default`, `'static'`, a constant backtick `class` literal,
`/static/.test(default)` and an ordinary `count` binding. The function requires
the exact member/literal types and boolean object value. Its full diagnostic
list must be empty. The invalid project changes only the boolean object value
to the string prop `default`; the actual full list is one TS2322 at App.vue
line 12, authored UTF16 column 56, severity 1, complete message
`Type 'string' is not assignable to type 'boolean'.`

Both projects declare required T1 execution. The ordinary test always runs;
T0 deferral must use only its exact package, binary and test name when present
at that PR head. Full T1 Cargo execution remains required.

## Verification and remaining work

Actual TypeScript 7.0.2 and real Vue 3.6.0-beta.10 observed both lists. The empty
invalid draft failed before the observed row was frozen. A temporary production
fault disabled shorthand key expansion while retaining its receiver rewrite.
The mutated library compiled and the valid SFC produced four real TS1005
syntax diagnostics, failing the full-list comparison. Restoring the source
byte-identically restored the expected lists. Normal first/repeat runs passed
all five prepared tests and ten projects; observed lists were identical.
Strict Clippy with warnings denied and targeted formatting passed.

This control does not rebuild the historical revision or prove all scanner
inputs. Escaped literals, interpolated backticks, regex character classes,
malformed expressions and broader SFC binding discovery still require their
own evidence. The production fault is absent from the prepared change.
Actions/full T1 queue proof, common differential admission, native L4 parity,
end ranges, related information and raw backend observations remain pending.
Native acceptance is zero. #6879 remains open.

## Exact T0 deferral

Stack this slice on #6941 after parent review. Extend only the present package
`vize_canon`, binary `fix_history_diagnostics`, exact test
`reserved_prop_shapes_preserve_literal_and_member_diagnostics`. Prior Canon/Corsa names
remain exact; source existence and T1 metadata are checked. Full Cargo T1 is
unchanged. Latest-head source Actions and exact-child full Check execution are
required; no queue before parent main/full proof. All native/history gaps stay open.
