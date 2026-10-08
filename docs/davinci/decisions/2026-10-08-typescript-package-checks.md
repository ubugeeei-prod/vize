# TypeScript package check helpers

Paired issues: [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision and source

Continue the [owned TypeScript migration](./2026-10-08-owned-typescript-migration.md)
with two independent package check helpers above its strict-checker provider.
The parent is PR [#8302](https://github.com/ubugeeei-prod/vize/pull/8302), exact
source `0bb7d231e70c318240337f35c74957488adde735`.

Move only these owned executable sources:

- `npm/marquette/scripts/check-size.mjs` to `check-size.ts`.
- `npm/builder/vite/type-tests/plugin-vue/check.mjs` to `check.ts`.

The first commit contains two 100% byte-exact moves. The next change types the
real-Vite command helper, updates the three owning package script references,
and includes both sources in the existing bounded strict native project.
Node APIs and erasable annotations suffice; no transpiler or new pipeline
stage is introduced.

## Preserved contracts

The Marquette helper retains all nine entries, their nine byte budgets, gzip
level 9, complete JSON output, and oversized-output refusal. Every generated
`dist/*.mjs` artifact and published export path retains its identity.

The real-Vite helper retains the producer pack, consumer `npm ci` flags,
declaration staging, installed-package export, and exact consumer TypeScript
compiler path and arguments. Its real Vite 8 lockfile and TypeScript consumer
fixture retain their original bytes. Native package preparation and
`npm/builder/vite/scripts/run-tests.mjs` belong to a separate provider graph.

Public package `engines.node` remains `>=22`. These scripts are private build
and verification tooling run under the already pinned workspace runtime.
Public JavaScript output is not renamed to TypeScript.

## Proof and delivery

Required proof compares all nine whole Marquette output packets against the
original parent blob on the same emitted bundles, checks oversized refusal,
and executes both original and migrated real-Vite consumer checks while
comparing their emitted declarations and original consumer source identity.
The existing native checker must accept the expanded eight-file strict
project, including its original positive and three compiler-refusal controls.
Owning package checks and exact-head source Actions remain required.

The child PR uses the parent branch as its base and must join the same native
GitHub Stack before protected queue admission. Root owns admission and final
merge tracking. A draft PR, local proof, or a green source check does not prove
protected delivery or publication. The remaining handwritten `.mjs`
migration remains unfinished.

## Local original-output comparison

Move-only commit `ae4108dfcd15fc389aed0e2b0faefc6cadf4215f` has no inserted or
deleted lines. Removing the real-Vite helper's three parameter annotations
recovers its complete original parent blob; the Marquette source is still
byte-exact without erasure. Only canonical decision row 225 changes among its
350 original rows. Both manifests retain every non-script key, including
public exports, package file lists and Node engines.

Under Node 24.14.0, `vp run --filter './npm/marquette' build` passes. Running
the original parent blob from the same package script directory and the
migrated script on those identical emitted bundles produces the same nine
whole JSON lines and empty stderr. Their combined stdout SHA-256 is
`36cdb81bc45fc71726bd00e024eae96f567d21bc8f6e5612a7a7cb1089648849`:

| Entry                                   | Gzip bytes | Original maximum |
| --------------------------------------- | ---------: | ---------------: |
| `@vizejs/marquette`                     |        523 |             1024 |
| `@vizejs/marquette/adapter`             |       2810 |             4096 |
| `@vizejs/marquette/validate`            |       2339 |             3072 |
| `@vizejs/marquette/test-run`            |        515 |             1024 |
| `@vizejs/marquette/test-run/validate`   |        157 |             4096 |
| `@vizejs/marquette/test-run/canonical`  |       1573 |             2048 |
| `@vizejs/marquette/test-run/admission`  |       2571 |             3072 |
| `@vizejs/marquette/test-run/check`      |       1854 |             3072 |
| `@vizejs/marquette/test-run/transition` |       3306 |             4096 |

Nine separate oversized controls replace each entry in turn with the same
16,384-byte deterministic buffer: concatenate SHA-256 digests of decimal
integers 0 through 511. All nine original and migrated executions reject,
with identical complete stdout prefixes and stderr after replacing only the
exact original executable path with its migrated path. These temporary
controls never change a tracked fixture or emitted package bundle.

Executing the original real-Vite helper from its own script directory and
then the migrated helper passes both complete consumer checks. All four
emitted `.d.mts` files remain byte-exact, as do the original consumer
`package.json`, `package-lock.json`, `tsconfig.json` and `vite.config.ts`.
The actual consumer resolves Vite 8.2.2, TypeScript 5.9.3 and plugin-vue 6.0.9;
the existing install and compiler arguments are unchanged.

The eight-file native project, its original compiler positive/refusal law,
both owning package checks and the focused six-file Vite+ check pass. The
complete local transcripts, declaration and fixture hashes are in the
worktree artifact `target/typescript-package-checks/receipt.json`, SHA-256
`ad40fa49fd5386438485cb5a44af9c6d72c1e4b1a33443c40cf6a4ac1cf192b9`.
Exact-head Actions and protected Stack delivery remain pending.
