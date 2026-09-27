# Component event tuple diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Historical fixes: `e65154535ad5fb8645330848ea61f3fa8a65575b` (#3085) and
`4d95a2ddda5f33550cd408b5fa759e80bf9b68d4` (#1512/#1521).

## Decision and scope

Preserve all SFC input bytes and the authored config of the four original
`component_event_arity.rs` projects: unresolved valid multi-argument handler,
resolved valid tuple, wrong first argument and native extra required argument.
Their historical handwritten Vue declaration is replaced by the pinned real
Vue package; this is not a claim of byte-identical full project dependencies.
The new test compares every production diagnostic without file/code filtering.

Add a wrong second-argument project by changing only the valid handler's
`path: string[]` to `path: number[]`. The original first-argument-only predicate
cannot independently prove the full tuple. Two derived inline projects wrap
the original #1512 script/template fragments in an SFC and supply a real child
with `test: [value1: string, value2: number]`. The invalid arrow sends its second
parameter to a string-only helper, proving actual contextual type checking.

All seven required T1 projects pin complete messages, severity, code, authored
UTF16 starts and returned order. The three valid lists are empty. Each invalid
project has one diagnostic:

| Project                        | Code   | App.vue start | Meaning                                  |
| ------------------------------ | ------ | ------------- | ---------------------------------------- |
| Wrong first argument           | TS2322 | 8:24          | string payload versus number handler     |
| Native extra required argument | TS2345 | 7:27          | DOM supplies one required argument       |
| Wrong second argument          | TS2322 | 8:24          | string[] payload versus number[] handler |
| Inline second argument         | TS2345 | 9:85          | contextual number versus string helper   |

Current component assignment diagnostics use TS2322 and the authored event
name following #3509, replacing the historical call-site TS2345 contract.
Unresolved modern listeners remain variadic; native handlers remain singular.
Nullable handlers and the replaced scanner are separate requirements.
The older #855 removed positive payload tests are coverage retirement, not
behavior retirement; restoring those exact typed/overloaded projects remains
open and receives no credit from these fixtures.

## Verification and remaining work

Actual TypeScript 7.0.2 / Vue 3.6.0-beta.10 observed all lists. Each initially
empty invalid draft failed before its actual row was frozen. A temporary
production fault collapsed a resolved modern listener tuple to its first
member while preserving unresolved fallback. The mutated library compiled;
the valid two-argument project then emitted an actual TS2322 at 8:24 with
`Expected 2 or more, but got 1.`, failing its empty-list assertion. The production
source was restored byte-identically. Restored normal first/repeat executions
passed all six prepared tests and seventeen projects, with identical tuple
observations. Strict Clippy with warnings denied and targeted formatting passed.

The runtime test is unconditional. Its exact package `vize_canon`, binary
`fix_history_diagnostics` and test
`component_event_tuples_preserve_all_argument_diagnostics` can leave T0 only
when present at that child head; full unfiltered required real-TSGO T1 is unchanged.
Parent review approved the bounded slice for a stacked draft. Compound/superseded
contracts, Vue 2, runtime/generic/model/fallthrough emit variants, common
admission, Actions/full T1 proof, native L4, end/related/raw fields and complete
history remain open. Native acceptance is zero and #6879 stays open.

## Exact T0 deferral

Stack this slice on #6953 after parent review. Extend only the present package
`vize_canon`, binary `fix_history_diagnostics`, exact test
`component_event_tuples_preserve_all_argument_diagnostics`. Prior Canon/Corsa names remain
exact; source existence and T1 metadata are checked. Full Cargo T1 is unchanged.
Latest-head source Actions and exact-child full Check execution are required;
no queue before CI parent main/full proof. All native/history gaps stay open.
