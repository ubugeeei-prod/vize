# Native lint execution configuration (#7248)

The shared config and Vite+ `lint.vize` accept `crossFile`, `crossFileTree`,
`crossFileComplexity`, `strictReactivity`, and `maxWarnings`. Cross-file settings
remain whole-run options, while strict reactivity is translated into the real
rule configuration so existing editor consumers and ordered scopes can apply it.
Explicit rule severity wins over the config switch. Direct CLI flags keep their
existing opt-in behavior, and CLI warning limits override config values.

Keep the published exhaustive `LinterConfig` and feature-flag structs unchanged.
A new non-exhaustive execution-settings type and additive loader preserve the
old loader's result shape and reuse a single config parse. Public config
normalization already retains these fields without an extra serialization pass.

Vite+ rejects native-only lint flags before launching either checker, directing
users to `lint.vize`. Generated TypeScript declarations and JSON Schema follow
the Pkl source. The linter schema and CLI config loader are extracted in a
separate move-only commit.

Validation uses owning-product Vue fixtures, full JSON report equality against
CLI cross-file flags, `--no-config`, configured warning failure and CLI override,
raw strict-reactivity rule activation/disable tests, and serialized Vite+ config
with an exact Oxlint argument vector. Required Actions must pass on the PR and
merge-group candidate before completion. No product provider is promoted and
no performance or differential budget is loosened.

The public-key audit follows `serde(flatten)` owners instead of counting the
private `execution` field as a serialized key. It includes all fields of the
typed execution options in each Pkl/schema/TypeScript comparison. Maestro reads
strict-reactivity rules today; its existing cross-file lint gate still logs an
unfinished analyzer integration. This change does not claim editor cross-file
lint parity with the native CLI.
