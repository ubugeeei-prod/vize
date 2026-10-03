# Authored unused-symbol diagnostic history

This bounded #6879 / #6849 slice registers the original
[`cd7156d28386e072953476fdbc354a963758dc89`](https://github.com/ubugeeei-prod/vize/commit/cd7156d28386e072953476fdbc354a963758dc89)
(#1271) user TS6133 requirement. It changes fixtures and mandatory tests only.
Whole-commit admission, native history comparison and default migration remain
unfinished.

The complete original test function and its complete project/snapshot helper
functions are retained with their original blob, UTF8 excerpt ranges and
SHA256 identities. The complete Canon change is retained as a source-bound
patch. Both original SFC and explicit tsconfig literals are copied byte for
byte; the provenance test verifies their original literal ranges. The original
project deliberately has no node_modules. New fixed pack metadata preserves
that absence instead of silently substituting the pinned real Vue package
used by the nine previous packs. Their inputs, options and expected diagnostics
remain unchanged.

The original test can skip an unavailable runtime or failed checker and
filters the diagnostics to one TS6133/message substring. Those assertions do
not establish a complete vector. The new required BatchTypeChecker test has
no optional runtime branch and compares every returned diagnostic in order.
Its six complete project cases are:

| Case                | Compiler options                              | Complete public diagnostic vector |
| ------------------- | --------------------------------------------- | --------------------------------- |
| Original locals     | Original explicit noUnusedLocals=true         | App.vue 3:7 TS6133 unusedLocal    |
| Inherited locals    | Original options in base.json                 | Same single diagnostic            |
| Disabled locals     | Inherited true, both flags explicitly false   | Empty                             |
| Parameters only     | noUnusedLocals=false, noUnusedParameters=true | helper.ts 1:22 TS6133 unused      |
| Default locals      | Both flags absent                             | Empty                             |
| Parameters disabled | Full typed helper retained, both flags false  | Empty                             |

The independently authored parameter control retains the whole exported typed
function, including its body. It proves the configured production parameter
option branch; it is not presented as a historical literal. The original
source/config, derived configs and helper are all hashed in the immutable
shared manifest. Changed dependency metadata is rejected even if an attacker
refreshes the pack and manifest hashes.

The existing actual test observer writes only after the full unchanged
assertion succeeds. Required Actions still reconcile the real archive receipt,
exact source/tree, executable hash, input hashes, successful non-skipped JUnit
identity and all four worker observations. Public exitCode/success/blockType
remain retained but explicitly unbaselined. Their presence is not a new exact
process-status oracle. History state remains not-admitted until fresh required
source-built Actions provides actual observations.

A separate genuine native Vue law consumes the same original primitive SFC,
retaining the original observation, selected Program, File, Component and
interpolation. It compares the entire raw full report, authored range, actual
same-session compiler options and source/config bytes. Enabled options require
an error; disabled/default options retain the actual severity-4 TS6133 hint.
Batch's empty disabled vectors are never substituted for those native reports.
The diagnosing process must already be reaped when the result is published,
and no projected file is materialized.

The typed function has a real typed native refusal because the current sole
File walk rejects annotated parameters. It does not gain a fabricated native
profile. The new pack keeps native adapters null and stays in the unsupported
acceptance denominator.

Remaining work includes complete raw/end/related diagnostic history contracts;
separate actual CLI and Batch-LSP execution; an actual unmapped generated
warning proving suppression rather than merely its absence; broader original
anchor collector branches (imports/refs/CSS/other template scopes); the whole
bundled historical fix; coherent source/config/import-graph snapshots; and
full #6879 history/native/default migration. No upstream protocol or checker
options are changed by this slice. No local Rust build or backend installation
is used; runtime proof belongs to existing source-built Actions and the
protected queue.
