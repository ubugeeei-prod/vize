---
title: Configuration
description: Share Vue settings from your existing Vite config and TypeScript project.
---

# Configuration

Keep Vue tooling settings in `vue:` in `vite.config.*` and TypeScript settings in
`tsconfig.json`. The CLI, Vite integration and editor extension read these project
files. No dedicated Vize configuration is needed.

<span id="vite-configuration"></span>
<span id="choose-which-features-to-adopt"></span>

See [Shared Vue configuration](./vue-configuration.md) for complete project examples.

## Keep your Vite+ configuration

Use the Vize helper around your existing configuration:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
};

const vueConfig = {
  compiler: { sourceMap: true },
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

`withVue` keeps the existing Vite+ options, plugins and hooks. Config objects,
functions, promises and async functions are supported. An inline `vue: { ... }`
section is equivalent. See [Shared Vue configuration](./vue-configuration.md) for
paths, scoped entries and composition.


## TypeScript and Vue template checking

Keep the existing TypeScript project, including its standard `extends` and
`references`. Put Vue template compiler options in `vueCompilerOptions`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "jsx": "preserve"
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}
```

Existing projects using `vue-tsc` can retain their TypeScript project layout.
`compilerOptions` remains the authority for TypeScript strictness; Vue-specific
options belong in `vueCompilerOptions`. See the [reference](./configuration-reference.md)
for the supported Vue options and explicit Vize overrides.

<span id="change-one-rule"></span>

### Lint Rule Options

Change individual rules with `vue.linter.rules`. See [Rule Options](../rules/options.md) for complete typed options.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { rules: { "no-debugger": "error" } },
}).withVue({
  linter: { rules: { "vue/no-v-html": "error" } },
});
```

## Ordinary Vite

Keep the existing Vite configuration and replace the Vue compilation plugin:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vue: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
  },
});
```

<span id="standalone-cli"></span>

## CLI and editor

Run commands from the target package root. The editor resolves each document's
project inside the selected workspace and reads its Vite and TypeScript settings.
A monorepo package keeps its own `tsconfig.json`, references and Vite configuration.
Use `--tsconfig` to select another TypeScript project explicitly.

```bash
vp exec vize check
vp exec vize lint src
vp exec vize fmt --check src
```

Native relative paths in the Vite configuration use the selected Vite `root`.
A relative `root` resolves from the configuration directory. Explicit CLI file
arguments and `--tsconfig` remain relative to the command's working directory.
CLI feature flags and explicit editor feature switches override shared settings.

## Migrate removed configuration

`vize.config.*` and top-level `vize:` configuration are no longer supported.
Move their native settings to `vue:` in `vite.config.*`, and TypeScript project
options to `tsconfig.json`. Remove dedicated configuration files after migrating
and checking the result. A leftover dedicated file alongside a Vite config is an
error, the same migration error as a dedicated-only project. `vize init` and
`vize setup` preserve the original files and explain the migration.

There is no Vize-specific `extends`. Import ordinary JavaScript objects and pass
them to `defineConfig(vpConfig).withVue(vueConfig)` to reuse settings. Standard
TypeScript `extends` and project `references` continue to work.

See [Shared Vue configuration](./vue-configuration.md), the
[configuration reference](./configuration-reference.md) and the
[compiler reference](./compiler-configuration-reference.md) for individual options.

<span id="detailed-reference"></span>

See [Experimentals](./experimentals.md) for optional project features.

> [!NOTE]
> The CLI and editor use the existing Vite and TypeScript project files. Migrate removed dedicated configuration before deleting it.
