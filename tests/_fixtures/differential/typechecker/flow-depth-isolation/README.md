# A long `v-else-if` chain must not exhaust control-flow analysis

Vize checks every template expression under a `v-else-if` branch inside a
guard that repeats the negated conditions of the earlier branches. Each guard
was emitted inline in the template body, so every later reference walked all
earlier guards. A chain of a few dozen branches pushed one generated body past
TypeScript's control-flow depth limit.

TypeScript then reports `TS2563` ("The containing function or module body is
too large for control flow analysis") on the generated body and disables flow
typing for every file it checks afterwards. Vize hides diagnostics in generated
code, so the visible symptom was implicit-`any` errors (`TS7006`, `TS7031`) and
unused `@ts-expect-error` directives in unrelated files that `vue-tsc` reports
clean.

The fix folds the guard blocks that one emission pass writes back to back
into a single `if` / `else if` chain, the shape `vue-tsc` emits: a block with
the same guard reopens the previous block, and the next branch becomes
`else if (<enclosing guard> && <own condition>)`. Each condition then appears
once, the generated text grows with the number of branches instead of its
square, and narrowing flows through the chain.

## Where it was found

Misskey (AGPL-3.0-only) at `810faa8e5db4bbf9d6d408a61df148559340ee99`,
`packages/frontend/src/components/MkNotification.vue`: a notification card with
one `v-else-if` branch per notification type. The real-project matrix reported
363 false positives across 95 files (341 `TS7006`, 17 `TS7031`, 3 `TS2578`);
none of them were in `MkNotification.vue`. The files here are an independently
written reduction of that shape.

## Expected diagnostics

Recorded with `vue-tsc` 3.3.11 on TypeScript 6.0.3 over these files copied to
`src/` (`tsconfig.json.txt` as `tsconfig.json`): no diagnostics.

Before the fix, `vize check` on the same files reported
`src/later.ts 2:24 TS7006 Parameter 'value' implicitly has an 'any' type.`
