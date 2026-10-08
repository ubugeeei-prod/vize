# Keep the CLI configuration helper independent of native bindings

Tracking issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

A read-only audit of the still-open n8n adoption PR at
`5c2a2cf3d837b6538a3330a4d7cc817e4b6f04e2` found a new contract: Vue template
linting now uses `vize lint` and imported package `vize.config.ts` objects.
The older `aa173be0` Oxlint projection remains a separate frozen acceptance
cohort; its successful replay does not qualify this new CLI configuration.
Upstream branch source remains requirement evidence outside this repository,
under the existing license decision. No upstream writes are authorized.

The current shared CLI settings retain 51 selected rules, three explicit
`ruleOptions`, an editor-ui warning override and six scoped `entries` rule
disables. Read-only source audit finds the rule IDs, option shapes and
config-directory scope matcher already supported. This is source inspection,
not whole-workspace execution. The audited requirement file is
`packages/@n8n/oxlint-config/src/configs/vize.ts`, SHA-256
`d7b32f5a7bbf42ba301220b2ffee9a1dc372c252ab062e2f71ae3790149e46b5`.

The native CLI evaluates a package config in a separate Node process, imports
its default object and serializes that object for Rust normalization.
`defineConfig` only preserves the supplied object, array or factory identity.
Importing the public `vize/config` entry previously loaded both the Vize NAPI
binding and OXC Transform before that pure helper could be used. Those native
products are not needed for this evaluation and their absence could prevent
an otherwise valid config from loading.

Load the Vize binding only when JavaScript structural normalization actually
runs. Load OXC Transform only when the JavaScript loader transpiles a
TypeScript config. Rust config parsing, native normalization, options, scope
resolution and error propagation keep their existing contracts.

Four installed-package regression scenarios exercise the freshly built
public `vize/config` export with dependencies that record and throw on native
loading. Imported object, array, synchronous/async factory identity and the
pure global-types helper must succeed without loading either dependency.
`resolveConfigExport` and JSON `loadConfig` must still expose the missing Vize
binding, and TypeScript `loadConfig` must still expose its missing transformer.
These controlled dependency failures establish the lazy-loading boundary;
they provide no native lint accuracy or timing credit.

Fresh exact-head Actions and protected delivery are required for this fix.
Remaining latest-CLI acceptance must execute current-source package imports,
all 51 rules/options, nine licensed Vue roots, all 19 scriptless originals,
editor warning/entry scopes and complete authored positive packets. The two
n8n-local Oxlint plugins, complete frontend configuration, retired/type-aware
rule gaps, full monorepo checker baseline and installed-public acceptance
remain unfinished. Existing direct Oxlint SDK scriptless limitations remain
recorded for the older contract; the new CLI plan does not repair that SDK.
No upstream timing claim or complete adoption claim is made here.
