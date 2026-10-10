# Vite project roots in standalone commands

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The native projection retains the selected Vite project's root as auxiliary
document identity under the reserved `__vizeProjectRoot` projection key. It
resolves a relative root from the discovered config's
directory. Stable `VizeConfig`, `TypeCheckerConfig`, and existing loaded-config
models keep their public fields. The formatter's non-exhaustive snapshot and a
new non-exhaustive project snapshot carry this identity from one evaluation.

With no input argument, `build`, `lint`, `fmt`, and `check` use this root. Check
continues to select its TypeScript program through the existing project resolver
and preserves package routing and input-scope validation. Explicit file/glob
arguments and `--tsconfig` remain relative to the invocation directory. Explicit
output paths retain their existing CLI meaning. Dedicated configurations and
`--no-config` retain their existing path and default behavior.

Vite-owned `typeChecker` paths and scoped `basePath` values resolve from the Vite
root; an omitted scoped base path means that root. Top-level ignores use the root
independently of `basePath`, preserving the native ignore contract. The pure
projection uses Node's path helpers and adds no dependency on Vite or its plugin
execution. A Vite export with an entry array uses the existing pure Rust public
normalizer before native deserialization, retaining global values and ordered
scoped entries without a native addon dependency in config evaluation.

The differential fixture `config/vite-root-8371` is exercised through the actual
CLI in `project_config_cli/root.rs`: source discovery, compiler output, scoped
lint diagnostics, ignored files, formatter writes, TypeScript program selection,
explicit overrides, dedicated precedence, and `--no-config`. A config export
records each evaluation, so the public commands also detect duplicate evaluation.
The TypeScript test requires Corsa through the repository's existing requirement
helper; Actions must supply the checked-out executable and Vue dependency.
