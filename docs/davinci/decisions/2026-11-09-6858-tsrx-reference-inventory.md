# TSRX reference test inventory

Companion of the [TSRX Vue profile record](./2026-11-09-6858-tsrx-vue-profile.md)
(issue #6858). It lists the tests of the pinned reference implementation that
the Vue profile has to answer, so coverage can be counted instead of claimed.

Pin: `tsrx-org/tsrx` at `b09820a9b6b6ed44e640be8cca89b3debbb23491` (tag
`@tsrx/vue@0.3.5`, with `@tsrx/core` 0.5.5). Counts are files at that commit;
`it`/`test` counts were read from the files named.

## In scope for the Vue profile

| Area                         | Reference location                                                                                  | Size                                                                                          | Vize destination                              |
| ---------------------------- | --------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | --------------------------------------------- |
| Parser and early errors      | `packages/tsrx/tests/utils/parser.test.js`                                                          | 481 tests                                                                                     | L1 span and diagnostic tests                  |
| Tokenizer and syntax corners | `packages/tsrx/tests/utils/` (33 files)                                                             | `tokenizer-context`, `closing-brace`, `tag-names`, `comment-utils`, `error-codes`, `patterns` | L1 grammar tests                              |
| Shared compiler behavior     | `packages/tsrx/tests/shared/compile.js`                                                             | 221 tests, run per target                                                                     | L2 lowering and mounted parity rows           |
| Shared error behavior        | `packages/tsrx/tests/shared/errors.js`                                                              | per-target error suite                                                                        | diagnostic parity rows                        |
| Shared runtime harnesses     | `packages/tsrx/tests/shared/runtime/` (17 files)                                                    | switch fall-through, JSX text whitespace, script bodies, spread/ref order, TSX expressions    | mounted parity rows                           |
| Vue code generation          | `packages/tsrx-vue/tests/` (2 files)                                                                | 46 tests in `basic.test.js`, type-only types                                                  | mounted parity rows                           |
| Vue runtime                  | `packages/vite-plugin-vue/tests/runtime.test.tsrx`                                                  | 53 tests                                                                                      | mounted parity rows                           |
| Vue bundler integrations     | `packages/{vite,rspack,bun}-plugin-vue/tests/`                                                      | 18 files                                                                                      | CLI and bundler integration tests             |
| Scoped styles                | `packages/tsrx/tests/fixtures/scoped-styles/`, `shared/scoped-styles*.js`, `analyze/styles.test.js` | 16 fixtures with expected output                                                              | style scoping (unsupported until implemented) |
| Type checking                | `packages/typescript-plugin/tests/`, `packages/content-mapper/tests/`, `shared/type-diagnostics.js` | 15 and 36 files                                                                               | Canon projection tests                        |
| Source mapping               | `packages/tsrx/tests/shared/source-mappings.js`                                                     | shared suite                                                                                  | L1 span and projection mapping tests          |
| Language service             | `packages/language-server/tests/` (19 files), `packages/vscode-plugin/tests/`                       | completion, closing tags, symbols, formatting                                                 | Maestro LSP tests                             |

## Out of scope

- React, Preact, Solid, Hono, Ripple and Octane code generation and runtime
  tests: other host profiles.
- `packages/prettier-plugin/tests/` (409 files): formatting is a separate
  product surface; TSRX formatting is not part of #6858.
- Server submodule lowering (`server-module-lowering.test.js`,
  `script-body-server.js`) and platform flags (`platform.test.js`): host
  transport and build-time selection, unsupported in the Vue profile for now.
- Editor packaging tests for Zed, Sublime Text, Neovim and IntelliJ.

## Specification and implementation disagreements

None is recorded yet. When a reference behavior differs from the
specification draft, it gets a row here with both behaviors and the issue that
decides it. Until a maintainer decides otherwise, the pinned reference behavior
is the Vue profile's behavior, because parity is measured against it.
