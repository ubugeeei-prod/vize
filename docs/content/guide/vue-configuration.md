---
title: Shared Vue configuration
---

# Shared Vue configuration

Keep native Vue settings in the `vue` section of `vite.config.ts`. The compiler,
CLI and editor use the same native option names. Start with the
[Vite+ integration](./vite-plus.md) to run build, check, lint and format tasks.

## One configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    version: "3",
    compiler: { sourceMap: true },
    typeChecker: { strict: true },
    linter: { preset: "essential" },
    formatter: { singleQuote: true },
    languageServer: { hover: true },
  },
  server: { port: 3000 },
});
```

`vue` uses the native `compiler`, `typeChecker`, `linter`, `formatter`,
`languageServer`, `globalTypes`, `files`, `ignores`, `basePath` and `lib` members.
It also accepts `version`, `vite`, `musea`, `experimentals`, `dialect`, `name` and `entries`. Compiler and formatter sections contain
options; they do not have an `enabled` member. Typechecker, linter and language
server sections retain their native `enabled` flag. Unknown canonical members
produce an error.

## Keep an existing Vite+ configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = { server: { port: 3000 } };
const vueConfig = {
  compiler: { sourceMap: true },
  formatter: { singleQuote: true },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

The helper accepts the existing Vite+ object, promise, function or async function.
It forwards Vite's environment to configuration functions and preserves plugins,
hooks and Vite+ fields. Vue plugin injection runs once. Reuse ordinary imported
objects; a Vize-specific config file or `extends` is unnecessary.

## Paths and scopes

Relative native paths in Vite configuration use the selected Vite `root`, which
itself resolves from the configuration directory. Explicit CLI file arguments
retain their existing working-directory semantics.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "packages/app",
  vue: {
    files: ["src/**/*.vue"],
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

Canonical entries currently accept only `basePath`, `files`, `ignores` and
`linter`. Other sections and `version` remain project-wide; placing them in an
entry is an error. The CLI's scoped linter policy retains its existing behavior.
Per-file editor scope support and additional scoped sections remain unfinished.

## TypeScript project settings

Keep TypeScript `compilerOptions`, standard `extends` and `references` in the
existing TypeScript project. Put Vue template options such as `strictTemplates`
in its `vueCompilerOptions`; the CLI and editor use this project configuration.
See [Configuration](./configuration.md#typescript-and-vue-template-checking).

## Editing configuration outside the workspace

The editor follows the Vite configuration's imported helpers and the TypeScript
project's configuration dependencies, including files outside the workspace.
If a reload fails, it keeps the last valid settings. Watch targets include that
accepted dependency graph and representable file targets observed during the latest
failed attempt; failed settings are never adopted.

Automatic external-file notifications require both LSP client capabilities
`dynamicRegistration` and `relativePatternSupport`, plus a successful watcher
registration. The client baselines below have different support:

| Client baseline | External configuration changes |
| --- | --- |
| [VS Code 1.107.1 / languageclient 9.0.1](https://github.com/microsoft/vscode-languageserver-node/blob/release/client/9.0.1/client/src/common/fileSystemWatcher.ts#L40-L65) | Eligible for automatic watches with both capabilities. |
| [Zed 1.16.1](https://github.com/zed-industries/zed/blob/v1.16.1/crates/lsp/src/lsp.rs#L812-L817) | Advertises both capabilities; GUI watch validation remains pending. |
| [Neovim 0.11.4](https://github.com/neovim/neovim/blob/v0.11.4/runtime/lua/vim/lsp/protocol.lua#L561-L566) | macOS and Windows advertise both by default. Linux defaults require the next normal language operation. |
| [Helix 25.07.1](https://github.com/helix-editor/helix/blob/25.07.1/helix-lsp/src/client.rs#L596-L599) | No relative-pattern support; retry on the next normal language operation. |
| [Vim / vim-lsp 474c6566](https://github.com/prabirshrestha/vim-lsp/blob/474c656659b5fb51eec6770309e0211c8aa49b5b/autoload/lsp.vim#L686-L694) | No dynamic file watches; retry on the next normal language operation. Project-buffer saves also notify Vize. |
| [Emacs 29.3 / Eglot 1.12.29](https://github.com/emacs-mirror/emacs/blob/emacs-29.3/lisp/progmodes/eglot.el#L3421-L3468) | Project-directory watches do not cover external targets; retry on the next normal language operation. |

A normal operation such as requesting hover or completion checks configuration
freshness again. This provides recovery when the client cannot watch an external
file; it does not provide an immediate update when another process changes it.
Zed's automated integration coverage currently uses protocol replay, so it does
not establish GUI behavior. Client capability overrides can also disable watches.

### Repairing a missing imported helper

A newly missing helper can become a watch target when Node reports a structured
local `file:` URL, or Vite 8 reports an explicit relative or absolute file request
with a filename extension. Creating or repairing such a target can trigger a
reload when the client supports the required watches.

A failed Vite bundle may report a missing file without listing every intermediate
helper. Editing an omitted helper is detected at the next normal language
operation. The reported missing file can still be watched automatically.

Vite 7 errors without a structured file target, bare package names, extensionless
requests, custom thrown errors, and ambiguous Vite specifiers containing `%`, `#`
or `?` require a retry on the next normal language operation. The editor continues
watching the graph it already knows; these errors do not establish a watch on a
new target guessed from the error message.

Each configuration factory runs once per load. Reloads check known files before
and after loading. Vite's public bundle-loading API can discover a new helper
after that helper has already changed, so a reload is not an atomic snapshot of
concurrent edits across several files. Newly discovered file observations are
confirmed on the next normal language operation.

Vite's bundle loader can report a symlinked helper's resolved target without the
symlink path used by its import. Edits to that known target are observed, but
retargeting only the symlink requires reloading the language server. Normal
language operations cannot detect an import route absent from that dependency
graph. Without installed Vite, Node's resolve hook also retains explicit local
file requests with filename extensions as watch-only paths.

## Migrate existing settings

Move top-level `vize:` and dedicated `vize.config.*` native settings into `vue:`.
Use `typeChecker`, `linter` and `formatter` for Vue tools. Vite+ task extensions
use `typecheck`, `lint.vue`, `fmt.vue` and `pack.vue`. The removed dedicated config
formats and Vize-specific `extends` are no longer loaded. Preserve reusable
settings as ordinary imported JavaScript objects. Keep TypeScript options in
`tsconfig.json`, including `vueCompilerOptions`.
