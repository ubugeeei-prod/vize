---
title: Configuration
description: Share Vize settings from your existing Vite config and TypeScript project.
---

# Configuration

Keep Vize settings in `vite.config.ts` and the TypeScript project in `tsconfig.json`. Defaults need no dedicated Vize configuration file.

> [!NOTE]
> Shared Vite config discovery for the native CLI/editor and init without a dedicated config are being prepared for the next release. Until that release, published native tools still require the existing dedicated format for custom shared settings. The [reference](./configuration-reference.md) documents that format.

## Project and editor defaults

These defaults are planned for the next release for projects without a dedicated `vize.config.*`. Published releases retain their previous behavior.

| Feature | No dedicated config, including `vite.config.*` | Existing dedicated config |
| --- | --- | --- |
| Vue JSX checking in `vize check` and `vize lsp` | Enabled unless explicitly disabled | Opt-in: `typeChecker.jsxTypecheck: true` |
| LSP document, range, and on-type formatting | Enabled unless a project or editor switch disables it | Existing opt-in behavior |
| Formatter style options | Existing defaults and authored overrides | Existing defaults and authored overrides |

For React-owned or mixed React/Vue JSX, set `vize.typeChecker.jsxTypecheck: false` in Vite config. Vize does not infer the owning framework from JSX syntax; Vue SFC checking remains available. See [JSX type checking](./jsx.md#type-checking).

Project defaults do not override explicit editor switches. The VS Code lint-only profile still disables formatting; `vp run editor:setup` explicitly enables it while preserving existing choices. [Editor setup](./vite-plus-editor.md) explains formatter ownership.

## Vite+ configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

| Location | Controls | Run |
| --- | --- | --- |
| `compiler` | Vue compilation | `vp dev`, `vp build` |
| `lint.vize` | Vue lint rules | `vp run lint` |
| `fmt.vize` | Vue formatting | `vp run fmt:check` |
| `typecheck` | Vue type checking | `vp run typecheck` |
| `pack.vize` | Library declarations | `vp run pack` |

Use `vp run check` for combined Vize tasks. Built-in `vp check`, `vp lint`, and `vp fmt` retain Vite+'s own behavior. Existing scripts can rename generated tasks to `vize:<name>`; see [task names and overrides](./vite-plus.md#tasks).

## Change one rule

Put Vue rules in `lint.vize.rules` and Oxlint rules in `lint.rules`. Use [Rule Options](../rules/options.md) for option objects and [the rule catalogue](../rules/all.md) for complete examples.

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

### Lint Rule Options

Use [Rule Options](../rules/options.md) for the complete typed option objects and bad/good examples. Unknown option fields are rejected.

## Choose which features to adopt

Set `compiler`, `typecheck`, `lint.vize`, or `fmt.vize` to `false` to disable that feature. Vize formats Vue files; Oxfmt handles other files. See [ownership and conflict handling](./vite-plus.md#lint-and-formatter-ownership).

## Ordinary Vite

Use the plugin for compilation and a top-level `vize` object for settings shared with the CLI and editor. Importing the plugin also supplies the Vite config types.

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vize: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
    typeChecker: { strict: true },
  },
});
```

## Standalone CLI

Install `vize` and run commands from the target package root. The CLI reads `vite.config.*` and your TypeScript project. Vite+ `compiler`, `typecheck`, `lint.vize`, and `fmt.vize` settings are translated for native commands too. Explicit native settings in top-level `vize` take precedence over that translation.

```bash
vp install -D vize
vp exec vize check
```

CLI discovery stops at the nearest `package.json`, `tsconfig.json`, or `jsconfig.json`. In a monorepo, run from the target package or use `vize.entries`, and select the editor's workspace folders explicitly.

For the next release, the editor assigns each document to its nearest project root within the selected workspace. A `package.json`, `tsconfig*.json`, `jsconfig.json`, `vite.config.*`, or `vize.config.*` establishes that root. Without a Vite or Vize config there, the fresh-project defaults apply; for example, `package/src/tsconfig.json` selects `package/src` as a separate project. Invalid config retains the editor's historical defaults. An explicit CLI `--config` remains an error if the selected file is missing or invalid.

With Vite config, `build`, `lint`, `fmt`, and `check` without an input argument use the selected Vite `root`. A relative `root` resolves from the config directory. Vite-owned `typeChecker` paths and scoped `basePath` resolve from that root; dedicated-config paths retain their config-directory base. Explicit CLI file/glob arguments and `--tsconfig` remain relative to the directory where you run the command.

Shared global ignores exclude files from CLI discovery and editor lint. An ignored file opened in the editor still receives parser, type and navigation diagnostics. Ordered ignore patterns, including `!` negation, retain their authored meaning.

## Optional dedicated configuration

An existing `vize.config.*` remains supported and takes priority over Vite config in the same directory. CLI `--config` selects a file explicitly. Direct plugin options and explicit editor feature switches override shared settings; `config: false` disables automatic plugin config loading.

## Detailed reference

The [shared configuration reference](./configuration-reference.md) covers file discovery, dedicated TypeScript/JSON/PKL examples, scoped entries, Vue type resolution, and native LSP/Musea settings. Use the [compiler reference](./compiler-configuration-reference.md) for compiler options and syntax modes, and [Experimentals](./experimentals.md) for opt-in features.
