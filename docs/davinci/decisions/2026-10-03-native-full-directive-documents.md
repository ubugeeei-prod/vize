# Native Full Vue directive documents without arguments

Date: 2026-10-03

Owner: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847)

## Decision

Extend the opt-in native Vue 3 template Doc consumer to complete nonempty
`DirectivePrefix::Full` names whose existing L1 `VueDirectives` result has no
argument. This source slice follows the genuine dynamic-head provider commit
`a143c9ff5409bea8db025716af3747acddea811f` in #7505.

The consumer requires the original `v-`, typed name and complete modifier run
to cover the head contiguously. Without an argument, the modifier span must
start exactly at the typed name's end and end at the original head's end.
Incomplete colon forms such as `v-bind:` and `v-bind:.prop` leave a source gap
and refuse construction. Empty Full names, missing shorthand arguments,
incomplete dynamic heads, recovered trees and foreign source slices retain
explicit refusals. SourceBlock bounds and UTF-8 boundaries check every borrowed
piece; no expression parser, semantic name classifier or extra stage is added.

This admits the existing L1 spelling for `v-if`, `v-for`, `v-show`, object
`v-bind`/`v-on`, custom Full heads, modifiers and optional values. Acceptance
describes complete source layout, without asserting directive or value validity.
Opaque JS/TS and loop text, quotes, entities, multiline content, modifier runs
and attribute order remain original source slices.

The retained native L1 parse already implements logical `v-pre` suppression.
The formatter consumes its original Text children, preserving nested raw
interpolation bytes. Both `v-pre` and `v-pre.foo` remain positive suppression
controls; ordinary interpolation without that native control still refuses.
Glyph does not emulate suppression or change the native parser's policy.

## Validation boundary

Three new projection laws check borrowed Full name/modifier pointers, forged
prefix/name/modifier/UTF-8/overflow ranges, empty names and incomplete colons.
Eight new integration laws pin whole flat/broken output, optional values,
Unicode/entities/comments/multiline preservation, native `v-pre` suppression,
mixed-head order and fixed points, generated CRLF, retained refusal observations
and foreign equal-byte source custody. Existing static/dynamic/plain laws remain
controls; earlier no-argument refusal fixtures now cover incomplete heads.

Local proof checks Rust formatting, diff hygiene and unchanged canonical Glyph
inventory generators. Fresh hosted affected Rust Clippy/tests and the protected
merge queue supply exact-source whole-workspace and instruction acceptance.
Historical artifacts and duplicate manual full campaigns provide no acceptance
credit. This dependent child is not individually auto-merged.

## Publication and provider integration

The published child #7518 has #7505's actual branch as its base. GitHub's
native `gh stack link --remote origin 7505 7518` refused registration because
#7505 was already queued for merge. Both PRs reported no Stack membership;
the queued parent candidate `80cc646225e00cf672b52f9db29821609542bb71`
remains untouched. Parent-base links alone receive no native Stack credit.

The provider actually merged at 2026-10-03T09:01:27Z as valid signed
`80cc646225e00cf672b52f9db29821609542bb71`. Its protected Check
`37110594067` passed full Rust/tooling/differential validation and all 100
instruction ceilings, with every benchmark identical across three executions.
The merged native runtime and dynamic-law source matches its reviewed head.

The child is replayed on that fresh `main`, preserving its incoming allocation
test and regenerating the owned canonical Glyph shards. Retarget it to `main`
and rerun exact-head Actions before the resulting independent source enters
its protected queue. The old parent-base campaign remains evidence only for
its original head. No child queue or actual-merge credit follows from replay.

## Remaining work

Interpolation and genuine embedded formatting, other Vue dialects, SFC
assembly, public formatter options and checked span-edit integration remain
unfinished. Default formatter replacement waits for
[#6882](https://github.com/ubugeeei-prod/vize/issues/6882) and complete native
corpus acceptance. Legacy product routes, fixture bytes, instruction budgets
and oracle policy remain unchanged.

The retained compiler Program cannot be handed directly to the pinned OXC
formatter: L1's ordinary parser retains parentheses, while OXC's formatter
requires its `parse_for_format` profile with parentheses disabled and may panic
on an incompatible AST. Genuine retained embed documents require a compatible
native consumer/provider without reparsing or using a legacy formatter helper.
This readiness finding adds no bridge or provider behavior in this slice.
