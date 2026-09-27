# Template definite-assignment diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Historical fix: `39bf60c0614c888ba82f962b944160ce839cf035` (#4239).

## Decision and scope

Register two required T1 fixture projects using the same exact batch diagnostic
contract as the [event-handler slice](./2026-09-27-typechecker-fix-history.md).
The ordinary Rust integration test always calls the actual production checker;
the T0 nextest profile must explicitly defer its exact test name, while full
T1 Cargo runs it with the installed runtime. No test-body early return or
runtime-absence success is added.

The first project contains the exact 18 authored SFCs and `support.ts` from
`template_definite_assignment/fixtures.rs`. It pins every returned diagnostic,
including severity, authored 1-based UTF16 start, code, message, file and order:

| File                         | Code   | Start | Message                                                |
| ---------------------------- | ------ | ----- | ------------------------------------------------------ |
| OptionsApiConditional.vue    | TS2454 | 11:19 | Variable 'paginator' is used before being assigned.    |
| ScriptAsyncCallback.vue      | TS2454 | 8:15  | Variable 'followedTags' is used before being assigned. |
| ScriptConditional.vue        | TS2454 | 7:13  | Variable 'paginator' is used before being assigned.    |
| ScriptNeverAssigned.vue      | TS2454 | 4:13  | Variable 'paginator' is used before being assigned.    |
| ScriptWatchImmediate.vue     | TS2454 | 9:13  | Variable 'paginator' is used before being assigned.    |
| TemplateStillChecked.vue     | TS2339 | 10:21 | Property 'nope' does not exist on type 'Paginator'.    |
| TemplateVForStillChecked.vue | TS2339 | 10:54 | Property 'nope' does not exist on type 'string'.       |

All seven records have error severity. The other eleven SFCs have no diagnostic
in this complete project list. Deferred template reads remain valid across
immediate watchers, conditional assignments, top-level await, async callbacks,
non-immediate watchers, early returns and never-assigned bindings. Script reads
retain TypeScript control-flow checks, and invalid template members remain
checked.

The second project uses the exact existing `TSX_CONDITIONAL_SFC` input. The
previous test checked generated shadow text only; this fixture runs the actual
checker over the SFC and requires an empty complete diagnostic list.

These two projects link the real installed Vue 3.6.0-beta.10 package, rather
than the earlier unit test's hand-written Vue declarations. TypeScript 7.0.2
produced the same seven original diagnostic tuples in the first project and
zero in the TSX project. This new profile does not claim the old stub fixture's
complete backend payload or Vue 2.7 parity.

## Verification and remaining work

The actual checker passed the two projects locally. The original three
event-handler fixture projects also remained green. Exact case identities are
checked so missing, duplicated or reordered fixture projects fail.

Actions, the full T1 merge-queue lane, end ranges, related information, common
differential execution and native L4 comparison remain pending. Native
acceptance is zero. The history ledger records this bounded local start-only
contract without claiming the whole #4239 patch is admitted. The separate
plain-script/no-shadow, Vue 2.7 and other constructor/generated-code contracts
still need their own diagnostic evidence. #6879 remains open.

## Exact T0 deferral

Stack this slice on #6935. Extend only the present package `vize_canon`, binary
`fix_history_diagnostics`, exact test `deferred_template_reads_preserve_script_diagnostics`.
The prior event name and three Corsa names stay exact. Source existence and T1
metadata are checked by tooling; full unfiltered real-TSGO T1 remains unchanged. Draft Actions
and CI parent main/full queue proof remain pending.
