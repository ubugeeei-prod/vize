# Vize UI Source Install Planner Contract

`@vizejs/ui-tooling` publishes schema version 1 of the source install dry-run
planner from `./source-install-planner`. The planner is a pure TypeScript
primitive for repository tooling; it does not read from or write to the
filesystem.

## Inputs

- `mode` is `"dry-run"`; mutating install modes are outside this foundation.
- `requestedFamilies` names the source-owned UI families requested by a caller.
- `sourceFiles` is the caller-provided set of known files, grouped by family
  name, with source path, destination path, source digest, and optional
  destination digest.
- `destinationRoot` is normalized once and joined with each relative
  destination path for machine-readable target paths.

## Output

- `schemaVersion` is `1`.
- `actions` are sorted by target path, destination path, source path, family
  name, and operation.
- Each action is one of `create`, `overwrite`, `skip`, or `conflict`.
- `diagnostics` are stable records with `code`, `familyName`, `path`, and
  `message`.
- `ok` is true only when no diagnostics exist and no action is a conflict.

## Safety

- Unknown requested families fail closed with no actions.
- Unsupported runtime modes fail closed with no actions.
- Source and destination paths must be relative file paths.
- Absolute paths, empty paths, NUL bytes, and `..` traversal segments are
  rejected before any writable action can be produced.
- Duplicate destination paths become conflict actions.
- Dry-run planning never mutates the destination root; filesystem discovery and
  writes must stay outside this primitive.
