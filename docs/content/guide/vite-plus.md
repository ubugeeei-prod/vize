---
title: Vite+
---

# Vite+

Use one `defineConfig` for Vite+, the Vize compiler, native typechecker, native
linter, formatter, and library declarations. Install `@vizejs/vite-plugin` and
your preferred compatible `vite-plus` version in the project:

```bash
vp install -D @vizejs/vite-plugin
```

Keep the integration settings in **`vite.config.ts`**; no separate Vize config is
required for these tasks. [Migrate an existing project](./migration.md) with literal
import/plugin diffs, or follow [Getting Started](../getting-started.md).

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

> [!IMPORTANT]
> The helper requires Vite+ 0.2.3 or later.

Vite+ is an optional peer dependency. Vize uses the project's installed version
and its configuration types; it never downloads or replaces Vite+. Earlier versions can mistake a custom
`defineConfig` for Vite+'s own and skip its generated tasks
([upstream fix](https://github.com/voidzero-dev/vite-plus/releases/tag/v0.2.3)).
Ordinary Vite projects can keep the usual plugin entry point.

## Configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";

const config = defineConfig({
  server: { port: 3000 },
  lint: {
    rules: { "no-debugger": "error" },
    ignorePatterns: ["dist/**"],
  },
  fmt: { ignorePatterns: ["dist/**"] },
}).withVue({
  lint: defineVizeLintConfig(
    ...flatConfigs.recommended,
    ...flatConfigs.ecosystem,
    {
      ignorePatterns: ["dist/**"],
      rules: { "no-console": "warn" },
    },
  ),
  fmt: { singleQuote: true },
  entries: [
    {
      files: ["src/admin/**/*.vue"],
      linter: { rules: { "vue/no-v-html": "off" } },
    },
  ],
});

export default config;
```

Keep the root object as an ordinary, Vue-free Vite+ config. `.withVue()` composes
the Vue flat lint layer with that root configuration, so the root
`no-debugger` policy remains active alongside `flatConfigs.recommended`,
`flatConfigs.ecosystem`, and the Vue-specific `no-console` rule. Use `lint` and
`fmt` for the native Vue tools, and keep `entries[].linter` for file-scoped
native overrides. The older `linter` and `formatter` names remain compatibility
aliases, but do not specify an alias together with its replacement. See [Shared
Vue configuration](./vue-configuration.md) for paths and scoped entries.

| Section | Owner |
| --- | --- |
| `vue.compiler` | Native compiler settings shared with the CLI and editor |
| `vue.typeChecker` | Native checker settings; TypeScript defaults come from `tsconfig.json` |
| `vue.linter` | Native Vue lint presets, rules and options |
| `vue.formatter` | Native Vue formatting options |
| `vue.languageServer` | Shared language-server features |
| `compiler` | Vite+ compiler plugin controls; `false` keeps the existing Vue compiler when `vue.compiler` is absent |
| `typecheck` | Vite+ native-check task controls; `false` disables the task unless `vue.typeChecker.enabled` explicitly enables it |
| `lint.vue` | Native lint task options, including `typecheck`, `locale` and `helpLevel` |
| Other `lint` fields | The installed Vite+ / Oxlint configuration |
| `fmt.vue` | Native formatter task options; `false` returns formatting to Oxfmt when `vue.formatter` is absent |
| Other `fmt` fields | The installed Vite+ / Oxfmt configuration; `ignorePatterns` also excludes native Vue formatting |
| `pack.vue` | Native declaration and map options |
| Other `pack` fields | The installed Vite+ / tsdown configuration |

Canonical `vue` settings take precedence over the corresponding task convenience
options. For example, `vue.linter.enabled: false` disables native lint. Keep
TypeScript strictness and template options in the existing TypeScript project:

```json
{
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["src/**/*"]
}
```

The typechecker runs once per task, including `check` when lint requests it.
The helper removes its integration fields before passing the remaining config
to Vite+. Other Vite+ options keep their upstream types, including future
additions. Objects, promises, functions and async configuration functions are
accepted, with Vite's environment forwarded to config functions.

### Reuse existing configuration

Import ordinary JavaScript configuration objects and add Vue settings through
the fluent helper:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./vite.base.ts";
import vueConfig from "./vue.settings.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

This preserves existing Vite+ options, plugins and hooks and injects the Vue
compiler once. There is no custom config `extends`, and dedicated `vize.config.*`
files and top-level `vize:` settings are retired. Standard TypeScript `extends`
and `references` in `tsconfig.json` remain supported.

The standalone `withVue(vueConfig, vpConfig)` helper remains supported. It is a
Vue-first convenience function, not an alias of the config-first `defineConfig`.
`withVize` remains a compatibility name for `defineConfig`; prefer the fluent
API for new configuration.

## Tasks

| Command | Work |
| --- | --- |
| `vp run check` | Native typecheck, native lint, Oxlint, and both formatter checks |
| `vp run check -- --fix` | Typecheck, fix lint, and write formatting |
| `vp run typecheck` | Native typecheck only |
| `vp run lint` | Native lint and Oxlint, plus optional native typecheck |
| `vp run lint:fix` | Fix through both lint engines |
| `vp run fmt` | Vize writes Vue formatting; Oxfmt writes other files |
| `vp run fmt:check` | Check both formatters without writing |
| `vp run build`, `dev` | Vite+ with the Vize compiler |
| `vp run pack` | Vite+ library bundling and configured native declarations |
| `vp run preview`, `test` | The corresponding installed Vite+ commands |
| `vp run editor:setup` | Recommend extensions and Vue editor defaults |

Pass paths after `--` to check, lint, and format tasks. Formatting also accepts
`--check` or `--write`. Put tool options in the corresponding config section.
Checks continue after diagnostics so one failing tool does not hide the others.
Generated tasks are uncached so fixes and configuration changes take effect.

> [!WARNING]
> Custom tasks use `vp run`. Built-in `vp check`, `vp lint`, and `vp fmt` do not
> invoke these tasks.

Built-in `vp build`, `vp dev`, and `vp pack` already use the
configured compiler or pack hooks.

Existing package scripts are preserved. A pre-existing `check` script causes the
generated task to be named `vize:check`; the same rule applies to other names.
Explicit `run.tasks` definitions take precedence. Rename or disable generated
tasks with the optional integration argument:

```ts
export default defineConfig(
  { run: { tasks: { deploy: "your-deploy-command" } } },
  { tasks: { check: "verify", preview: false } },
);
```

`tasks: false` disables all generated tasks.

## Lint and formatter ownership

Vize's native linter runs alongside Oxlint, including Oxlint's JS/TS diagnostics
inside Vue files. No `oxlint-plugin-vize` registration is needed. Overlapping Vue
rules are disabled in Oxlint by default, using the installed tool's rule catalog.
Explicit `lint.rules` and overrides can re-enable a rule.

Vize formats `**/*.vue`, which is excluded from Oxfmt by default. Other files stay
with Oxfmt. Set `lint.vue: false` or `fmt.vue: false` to return the corresponding
responsibility to Vite+. The integration option `conflicts: false` disables
automatic overlap handling.

`fmt.ignorePatterns` applies to both formatters, so generated or vendored Vue files
stay untouched. To use `lint.vue: false` or `fmt.vue: false`, omit the corresponding
shared native settings that would explicitly enable that tool. `lint.vue.locale` (`en`, `ja`, or `zh`) and `helpLevel` (`full`,
`short`, or `none`) control native diagnostics without passing unsupported flags
to Oxlint.

The compiler replaces an existing `vite:vue` plugin, including nested and async
plugin lists, to avoid compiling a Vue file twice. Move native options to `vue.compiler` and plugin-only controls to `compiler`.
To retain the original compiler, omit `vue.compiler` and use `compiler: false`.

## Library declarations and maps

```ts
export default defineConfig({
  pack: {
    entry: ["src/index.ts"],
    outDir: "dist",
    format: ["esm"],
    vue: {
      dts: true,
      declarationMap: true,
      sourcemap: true,
    },
  },
});
```

Vue library bundling uses Vize's Rolldown integration. Providing `pack.vue`
enables native declaration generation by default; `dts: false` opts out.
Declarations are emitted after a successful bundle, and type errors fail the
pack command. Native declaration generation replaces tsdown's declaration pass
for that pack entry, so it understands Vue SFCs.

`declarationDir` overrides the output directory, otherwise `pack.outDir` is used.
`tsconfig` selects a TypeScript project. `declarationMap` overrides that project's
map setting without modifying authored tsconfig files. `sourcemap` controls
compiler and bundle maps. Each entry in a `pack` array can configure `vue`
independently; `pack.vue: false` leaves that entry with Vite+.

## Editor setup

Run `vp run editor:setup` to add the Vize and Vite Plus extension recommendations
and Vue-specific editor defaults. It preserves comments, existing recommendations,
explicit settings, and the formatter chosen for other languages.

See [Vite+ editor setup](./vite-plus-editor.md) for the recommended responsibilities
and how to share native configuration with the editor.
