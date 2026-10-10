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
For array exports, the final unscoped tool override retains the selected host
root, so an authored extension key in an earlier entry cannot redirect the CLI
to a different project.

Ignore strings remain authored strings, including ordered `!` negation and
escaped glob metacharacters. Global ignore projections carry the trusted Vite
root as their separate `base_path`; they are never made absolute with
`path.resolve`. CLI project collectors and editor scopes evaluate ordered
sequences within each base path, even when an absolute pattern outside the
project occurs between a positive pattern and its subsequent negation.
Dedicated native files discard the reserved
host identity before projection, so previously ignored extension values cannot
change their invocation/config-directory behavior. Existing dedicated collectors
retain their established matching rules. Authored absolute patterns are rebased
without interpreting glob syntax; ordered in-project negation still applies.
Default project globs discard their leading `./` before joining the root so
formatter discovery cannot succeed without actually formatting its selected files.

The differential fixture `config/vite-root-8371` is exercised through the actual
CLI in `project_config_cli/root.rs`: source discovery, compiler output, scoped
lint diagnostics, ignored files, formatter writes, TypeScript program selection,
explicit overrides, dedicated precedence, and `--no-config`. A config export
records each evaluation, so the public commands also detect duplicate evaluation.
The TypeScript test requires Corsa through the repository's existing requirement
helper; Actions must supply the checked-out executable and Vue dependency.

Shared global ignores exclude standalone CLI file discovery and editor lint
policy. Files explicitly opened in the editor continue to receive parser, type,
and navigation results, preserving the existing dedicated-config behavior and
TypeScript editor exclude convention. CLI and editor file selection are therefore
not identical; full public diagnostic vectors test the supported boundary.

The root fixtures compare every generated directory and complete compiler byte
output, full lint/check JSON, and the complete formatter project inventory.
Temporary project identity and unordered lint file enumeration are the only
report normalizations. Compiler snapshots retain the existing Vue output,
including whitespace preservation and custom-element lowering. Lint snapshots
retain essential-preset component-name diagnostics independently of alt-text
overrides. Check snapshots retain all program roots, compiler options, selected
files and TypeScript diagnostics, including the numeric assignment's TS2322.
The ignore-syntax fixture independently proves relative, absolute, negated and
escaped patterns: dropped/literal files remain untouched, explicit keep files
remain selected, and their authored TypeScript errors remain visible. Snapshots
were reviewed against these contracts before adoption; a formatter no-op caused
by an interior `./` in a root glob was corrected instead of recorded as expected.

Checked discovery also reports config-evaluation failures instead of silently
checking with unrelated defaults. The Actions app gate builds the lightweight
public CLI JavaScript exports before invoking the freshly built native CLI, so
workspace samples importing `defineConfig` from `vize` have the same available
exports as an installed package. The failing unbuilt-export receipt remains
evidence of that prerequisite; config errors and their diagnostics stay strict.
