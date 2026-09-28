# Shared project model, first path slice (#6874)

## Audit

- CLI `check` loads `vize.config.*` in `runner/direct.rs`, chooses an explicit
  `--tsconfig` or `typeChecker.tsconfig`, and then resolves the invocation
  project in `runner/invocation.rs`. It separately interprets the config file's
  directory for runtime and global-component paths.
- LSP `ServerState::load_workspace_config` and `load_lsp_config` each load the
  config and derive the checker settings. `workspace_folders.rs` also loads a
  linter context per folder. These are distinct callers of the Carton loader;
  a single parsed configuration shared across all product entry points is not
  implemented yet.
- LSP `state/corsa.rs` and `state/batch_cache.rs` each independently turned a
  relative `typeChecker.tsconfig` into a path below the workspace root. The
  CLI had a third path calculation relative to the config file. The Corsa
  session cache also searches for a nearest `tsconfig.json` per source; CLI
  `tsconfig_inputs` resolves reference graphs and file ownership separately.
- LSP primary-root selection (`rootUri` then first workspace folder) and
  deepest-folder lint policy are in `workspace_folders.rs`; CLI project-root
  and nearest-tsconfig selection are in `runner/resolve.rs`. They have distinct
  inputs and remain distinct in this slice.

## Decision

`vize_l0::config::ProjectModel` is a small owned snapshot of root, config
source path, and configured tsconfig path. It resolves relative config paths
against the config file's directory, then the invocation/workspace root when
there is no source. An explicit CLI tsconfig is relative to the invocation
root and wins. CLI `check` and the LSP Corsa and batch-checker initializers
now use this model for the same path rule. The model does no source scan, owns
no TypeScript session, and introduces no new pipeline stage or serialization.

This is a foundation slice, not completion of #6874. A subsequent slice must
move config loading, workspace-folder selection, effective tsconfig ownership
and invalidation behind one project model without changing existing product
output. Keep the model request/folder scoped; do not add per-node queries,
per-node refcounting, or whole-workspace resident state. Actions must verify
the CLI and LSP matrix before any claim of parity or issue closure.
