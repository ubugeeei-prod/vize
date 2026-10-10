# Project configuration discovery

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

Vite and TypeScript project files should suffice for the compiler, linter,
formatter, checker, editor, Musea and source-library workflows. Dedicated Vize
configuration remains compatible. Initializers preserve user files and do not
create a dedicated configuration.

## Shared configuration

An ordinary `vite.config.*` carries native settings in a top-level `vize`
section. Vite+ configurations additionally project `compiler`, `typecheck`,
`lint.vize`, and `fmt.vize`. Common `fmt.sortImports` also applies to Vize;
other native formatter options belong under `fmt.vize`, preserving the existing
Vite+ task payload and formatting history. Fresh native check tasks enable JSX
without adding unrelated checker defaults to formatter or lint payloads.
Native settings include scoped entries, globals, editor features, Musea and the
source library. TypeScript options and references remain in `tsconfig.json`.

The dependency-free projection lives in
`crates/vize_carton/src/config/loader/vite-runtime.mjs`. Native loaders embed it
from their publishable crate; the npm CLI bundles the same module. Published
consumers need no repository-relative files. The CLI adds no Vite runtime
dependency. Evaluation uses the existing Node configuration path. The Vite+
factory's source symbol permits reading settings without starting its plugins
or generated tasks.

## Precedence and roots

1. An explicit config file wins. `--no-config`, loader `mode: "none"`, and plugin
   `configMode: false` retain their opt-out behavior.
2. Within one directory, dedicated configs retain their previous order: Pkl,
   TypeScript, JavaScript, ESM, JSON. They precede Vite files.
3. Vite discovery uses TypeScript, JavaScript, ESM, CommonJS, MTS, CTS.
4. With no dedicated file, project defaults enable standalone JSX checking.
   Dedicated files retain their historical feature defaults.

Automatic CLI discovery ascends until the nearest package, TypeScript or
JavaScript project boundary. An unconfigured monorepo package does not inherit
a parent or sibling policy. Explicit roots stay shallow, matching LSP
workspace-folder isolation and npm `mode: "root"`. Relative native paths belong
to the selected config's directory; explicit CLI paths belong to the invocation.

Vite hooks use the already evaluated configuration with a dedicated-only disk
lookup, avoiding a second import of the active Vite file. Explicit plugin
options retain their precedence over shared settings. Vite+ retains explicit
tool-section overrides over inherited native settings.

Fresh editor projects expose formatting with the existing recommended editor
profile. Project `languageServer`/`lsp` switches override defaults; explicit
editor switches override project settings. Dedicated projects retain formatter
opt-in behavior.

## Qualification and remaining boundaries

Public CLI/LSP, packaged initializer and editor regressions qualify customized
settings, defaults, positive/negative controls, explicit overrides, dedicated
compatibility and monorepo package boundaries. Combined source Actions and
protected queue qualification are required before actual delivery.

The LSP retains one process-wide capability/type-checker profile. Root scoped
entries and explicit workspace folders provide monorepo policies. Automatic
per-document adoption of separate nested Vite configs remains unfinished.
Vite `root` selects the bundler root; standalone source selection still follows
the invocation and explicit patterns/TypeScript project. Root parity remains
unfinished without a source-bound public regression. This change does not
close #8371's complete acceptance until these remaining boundaries are resolved.
