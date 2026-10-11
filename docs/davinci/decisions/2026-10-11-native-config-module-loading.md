# Native configuration module compatibility

The v0.445.0 App E2E run [38111801880](https://github.com/ubugeeei-prod/vize/actions/runs/38111801880)
found that the native configuration loader treated authored Vite settings as raw
Node ESM. Misskey's original JSON import consequently failed under Node 24.14.0.
Naive UI also uses an extensionless TypeScript helper and `__dirname` in `.mts`;
VOICEVOX imports a `.js` path whose authored helper is TypeScript. Installing the
projects' missing dependencies alone does not give Node Vite's loading semantics.
This correction belongs to [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The native loader uses the selected project's installed Vite through its public
`loadConfigFromFile` API and its existing `bundle` loader. Vite owns transformation,
JSON handling, relative imports and the original per-file scope variables. See
[Vite configuration loading](https://vite.dev/config/) and the
[public API source](https://github.com/vitejs/vite/blob/v8.0.16/packages/vite/src/node/config.ts).
The selected project supplies its own Vite version and dependency graph; Vize does
not substitute another project's plugins or install missing imports.

A temporary adapter exports the original module namespace inside an object.
Vite evaluates that object, preserving the original default factory and Vize's
source symbol. The existing native projection then evaluates the actual source
once, without starting Vite, plugins or tasks. The adapter and Vite's temporary
output are confined to a private OS temporary directory and removed on both
success and error. The authored configuration is not rewritten, and the loader's
selected source path remains the original file.

Dependency-free configurations retain native Node loading when the project's
normal package search has no installed Vite. A present but broken Vite package,
missing import, malformed JSON, or thrown configuration error remains an error;
the bundle loader never falls back to native loading after a failure. No Node
module hooks or newer hook API requirement are introduced.

The regression corpus uses the production module loader with the installed real
Vite. It covers all four compatibility cases, original file scope, raw factory
authority, exact single evaluation, malformed JSON and missing imports, broken
installed Vite, dependency-free settings, and temporary-file cleanup. Original
public native configuration tests and failure contracts remain in force.

The full original Misskey configuration exposed a version-specific requirement:
Vite 7.3.1's esbuild dependency externalizer treats a `file:` adapter import as
external and therefore loads the original TypeScript through raw Node ESM.
The adapter imports the authenticated absolute filesystem path instead, so the
original module enters Vite's bundle graph. The complete source and file scope
remain original. A frozen Vite 7.3.1 fixture adds this regression to the corpus;
the original adapter fails its JSON import and the corrected adapter passes.
The eight laws retain the installed Vite/Vite+ coverage and all refusal controls.

The minimal corpus is separate from full original-project acceptance. The
original pinned Misskey, Naive UI, npmx and VOICEVOX configurations still require
their complete dependency closures and factory environments. Those full rows,
the exact-head Actions, protected merge and subsequent release must succeed
before this release blocker is considered delivered.

The original paired source clauses are retained below after the decision
collection became canonical. They describe their originally qualified versions.

**2026-10-11 native configuration module compatibility (#8371):** Use the selected project's installed Vite public bundle loader through a temporary namespace adapter; retain the original config and raw source factory, one evaluation, selected-file authority, and fail-closed imports/errors. Dependency-free configs retain native Node loading; broken installed Vite never falls back. Seven real-loader/embedded-script laws cover JSON, extensionless and JS-to-TS helpers, file scope, factory authority, errors, and cleanup. Full pinned-project/Actions/merge/release acceptance remains required; see [decision](./2026-10-11-native-config-module-loading.md).

**2026-10-11 Vite7 original-module correction (#8371):** Full unchanged Misskey exposed esbuild externalizing a file: adapter URL under its ownVite7.3.1. Import the authenticated absolute filesystem path into the same bundle graph, preserving original file scope and raw factory. A frozen genuineVite7.3.1 corpus fails with the initial adapter and passes with this correction; all eight loader/embedded-script laws pass with original refusal controls. Full original-project and exact-headActions/protected/release qualification remains separate.
