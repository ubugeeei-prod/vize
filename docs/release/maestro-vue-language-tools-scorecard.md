# Maestro Vue Language Tools Scorecard

This evidence file closes the release-blocking #3224 dimensions that were still open for Maestro
editor parity against `vuejs/language-tools` / Vue Language Server.

The source of truth is `tests/_fixtures/maestro-vue-language-tools-scorecard.json`. It is checked by
`tests/tooling/lsp-vue-language-tools-scorecard.test.ts`, which fails if a required dimension,
must-include oracle, must-exclude oracle, editor evidence row, or latency budget loses its executable
backing.

## Feature Matrix

The gated dimensions are diagnostics, completion, signature help, hover, definition, references,
rename, code actions, semantic tokens, inlay hints, document features, file rename, and workspace
symbols.

Each row in the fixture has:

- `lspMethods`: the protocol requests or notifications covered by the row.
- `mustInclude`: behavior that must be present, such as authored-range diagnostics, script binding
  completion, template hover, declaration navigation, quick fixes, semantic token tuples, and
  workspace symbol indexing.
- `mustExclude`: behavior that must stay absent, such as stale diagnostics, context-leaking
  completions, non-renamable directive edits, unrelated code-action kinds, plain-text semantic token
  leakage, stale document structure, external rename rewrites, and stale workspace symbols.

`tests/tooling/lsp-vue-language-tools-oracles.test.ts` also executes a representative real
`vize lsp` session that checks positive and negative oracles across the matrix, rather than relying
only on metadata.

## Editor Breadth

CI-backed editor evidence is declared in the scorecard fixture and checked against the real workflow
and task graph.

| Editor  | Evidence level                                                       | CI task                            |
| ------- | -------------------------------------------------------------------- | ---------------------------------- |
| VS Code | Packaged extension host against the real server                      | `test:vscode-extension:host-real`  |
| Zed     | Official extension CLI validation plus real-server protocol scenario | `test:zed-extension:real-server`   |
| Neovim  | Packaged headless real-server scenario                               | `test:nvim-extension:real-server`  |
| Helix   | Official `hx --health` plus real-server protocol scenario            | `test:helix-extension:real-server` |
| Vim     | Packaged vim-lsp real-server scenario                                | `test:vim-extension:real-server`   |
| Emacs   | Packaged Eglot command/profile/archive spec                          | `test:emacs-extension:headless`    |

Emacs is intentionally labeled as packaged Eglot evidence, not real-server E2E, because there is no
scripted headless Eglot scenario in this repository yet.

## Latency Budgets

Misskey and Vue Vben Admin latency gates are enforced from
`tests/_fixtures/vue-ecosystem-fixtures.json` by `tests/performance/misskey-lsp-incremental.test.ts`
and `tests/performance/vben-lsp-incremental.test.ts`.

The scorecard fixture names the required lanes explicitly:

- Completion: `completion`.
- Hover: `hover`.
- Diagnostics to stable: cold open, warm no-op, leaf break/repair, shared dependency
  break/repair, plus second-app and cancellation convergence lanes for Vue Vben Admin.

`.github/actions/check-vue-parity/action.yml` runs these suites through
`test:performance:lsp-incremental` and uploads their metrics artifacts for review.
