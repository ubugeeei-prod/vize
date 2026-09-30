# Davinci implementation

This directory contains the native level crates, conversion crates and shared
infrastructure. Legacy products and transitional product adapters live in
[`../crates`](../crates).

`crates/` may depend on `davinci/`. Normal and build dependencies from `davinci/`
back into `crates/` are forbidden; dev-only differential oracles are permitted.
GitHub Actions checks declarations, including inactive optional dependencies,
target-specific dependencies and transitive workspace helpers.

The level restructure remains in progress. See the
[decision record](../docs/davinci/decisions/2026-09-27-level-restructure.md) and
[directory boundary](../docs/davinci/decisions/2026-09-30-davinci-directory.md).
