---
title: Vite+ editor setup
---

# Vite+ editor setup

Keep Vite+'s Oxc and Vitest integrations, and add Vize for Vue. The recommended
division is Vize for Vue typechecking, native lint, language features, and Vue
formatting; Oxc continues linting JS/TS (including script diagnostics in SFCs) and
formatting other files. The [Vite+ config helper](./vite-plus.md) disables
overlapping Oxlint rules and excludes Vue from Oxfmt.

## VS Code

Run `vp run editor:setup`, then install the recommended workspace extensions.
The task adds `VoidZero.vite-plus-extension-pack` and `ubugeeei.vize` to
`.vscode/extensions.json`. Existing recommendations and comments are retained.
It adds missing Vize capability settings and these Vue-specific defaults:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": true,
  "vize.editor.enable": true,
  "vize.ecosystem.enable": true,
  "vize.formatting.enable": true,
  "[vue]": {
    "editor.defaultFormatter": "ubugeeei.vize",
    "editor.formatOnSave": true
  }
}
```

The task preserves explicit settings, including `false` and an existing Vue
formatter. Review those existing choices if a provider does not activate.
It never changes the global formatter or disables Oxc. Keep the JavaScript and
TypeScript formatter settings from [Vite+'s IDE guide](https://viteplus.dev/guide/ide-integration).

Use `Vize: Show Status` to inspect active features, select the language-server
binary, or restart the server. The native language server must be available;
the extension's binary discovery/download flow handles this separately from the
project's Vite+ version.

Choose one Vue language-service owner. For Vize's full editor profile, disable
Vue - Official in this workspace to avoid duplicate Vue type diagnostics and
completion providers. To keep Vue - Official, use `Vize: Enable Lint-Only Profile`,
leave Vize typechecking/editor features disabled, and choose one Vue formatter.
Oxc and Vitest remain enabled in either profile.

Avoid a broad `source.fixAll` save action that invokes every installed provider.
Keep Vite+'s explicit `source.fixAll.oxc` action; use Vize's diagnostic quick fixes
for native rules. Vue formatting is selected with the language-specific
`editor.defaultFormatter` setting above.

## Native editor configuration

The editor's native language server discovers `vize.config.*`; it does not
evaluate `vite.config.ts`. Inline `lint.vize`, `fmt.vize`, and `typecheck` settings
configure the helper's tasks. To share those settings with native editor
consumers, keep common native settings in a supported `vize.config.*` file:

```json
{
  "linter": { "preset": "essential" },
  "formatter": { "singleQuote": true },
  "typeChecker": { "strict": true }
}
```

The helper reads an existing native config automatically when its `vize` section
is omitted. Task-specific inline sections override those shared settings.
JSON or Pkl is the simplest common format for the native language server.
Keep this file as the source of shared editor/task rules instead of copying the
same rule list into editor settings.

## Other editors

Keep Oxc's integration for non-Vue files and select Vize's language server for
Vue. Select a single formatter for Vue; keep the Vite+ `fmt` configuration for
other languages. Zed users can retain Vite+'s Oxc setup and select the Vize server
for `Vue.js`. The same shared native config keeps Vize diagnostics consistent
across editors.
