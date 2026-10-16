# Public configuration module loading

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The native CLI/editor loader already uses the selected project's installed
Vite bundle loader. The public `vize/config` loader instead transformed only
the selected TypeScript file and imported a temporary ESM module. A normal
`vite.config.ts` importing `./settings` from `settings.ts` therefore worked in
the native loader but failed with `ERR_MODULE_NOT_FOUND` in the public API.
Nuxt's project lint rules consume that public API too.

Use the existing canonical module loader for selected Vite configuration files
in both callers. Its namespace adapter preserves the original default export
and Vite+ source symbol: the bundler does not call the user's config factory.
The existing projection invokes that factory once with the caller's environment.
Vite module resolution remains relative to the selected config, including the
project's Vite version. A broken installed Vite or an invalid selected config
fails; neither permits a global loader or another config to be selected.

The canonical loader accepts a caller-owned fallback when Vite is absent. Its
native default remains a direct Node import. The public API falls through to
its existing TypeScript transform or fresh direct import. Dedicated Vize files,
discovery order, explicit selection, normalization and project roots retain
their previous behavior.

`public-vite-module-8371/project.json` retains the complete minimal project.
The source and packed public APIs compare the entire normalized result with
the native module loader over those unchanged files. The real Nuxt project-rule
consumer reads the same original project through `vize/config`. Additional
controls cover JSON and TypeScript helper imports, file scope, source/factory
evaluation counts, environments, dedicated precedence, explicit selection,
dependency-free loading and failure without fallback. Existing canonical loader
controls retain real Vite 7 and Vite 8 execution and adapter cleanup.

This bounded module-loading correction does not complete #8371. Full installed
fresh/migrated build, lint, formatting and checker/editor parity remain separate
acceptance work. The current packed initializer matrix exercises ten manager
cells and a workspace checker/LSP cell, but its shapes disable lint and do not
execute generated formatter/build scripts. A passing PR job that explicitly
sets `VIZE_TEST_DISABLE_TSGO=1` does not prove its skipped native checker cases.
