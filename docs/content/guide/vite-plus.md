---
title: Vite+
---

# Vite+

Use one `defineConfig` for Vite+, the Vize compiler, native typechecker, native
linter, formatter, and library declarations. Install `@vizejs/vite-plugin` and
your preferred compatible `vite-plus` version in the project:

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

Vite+ is an optional peer dependency. Vize uses the project's installed version
and its configuration types; it never downloads or replaces Vite+. The helper
requires Vite+ 0.2.3 or later because earlier versions can mistake a custom
`defineConfig` for Vite+'s own and skip its generated tasks
([upstream fix](https://github.com/voidzero-dev/vite-plus/releases/tag/v0.2.3)).
Ordinary Vite projects can keep the usual plugin entry point.

## Configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  typecheck: { strict: true },
  lint: {
    vize: {
      preset: "essential",
      rules: { "vue/no-v-html": "error" },
      typecheck: true,
    },
    rules: { "no-debugger": "error" },
    ignorePatterns: ["dist/**"],
  },
  fmt: {
    vize: { singleQuote: true },
    ignorePatterns: ["dist/**"],
  },
  server: { port: 3000 },
});
```

| Section | Owner |
| --- | --- |
| `compiler` | Vize compiler and plugin options; `false` keeps the existing Vue compiler |
| `typecheck` | Native typechecker options; `false` disables native checks |
| `lint.vize` | Native lint rules and optional `typecheck`; `false` disables native lint |
| Other `lint` fields | The installed Vite+ / Oxlint configuration |
| `fmt.vize` | Native Vue formatter options; `false` returns Vue formatting to Oxfmt |
| Other `fmt` fields | The installed Vite+ / Oxfmt configuration |
| `pack.vize` | Native declaration and map options |
| Other `pack` fields | The installed Vite+ / tsdown configuration |
| `vize` | Shared native config, including scopes, globals, and language-server settings |

`vize.lint.typecheck` is also supported. When both spellings are provided,
`lint.vize` wins for overlapping properties. The typechecker runs once per task,
including `check` when lint also requests it.

The helper removes Vize-specific fields before passing config to Vite+.
Other Vite+ options keep their upstream types, including future additions.
Promises and async configuration functions are accepted.

Use `extends` for shared presets. Bases are merged in order, followed by the local
configuration, using Vite+'s configuration merge rules:

```ts
import base from "./vite.base.ts";

export default defineConfig({
  extends: base,
  lint: { vize: { typecheck: true } },
  fmt: { vize: { tabWidth: 4 } },
});
```

A base can be a plain config, promise, config function, another `defineConfig`
result, or an array of these. Cycles produce an error. `withVue` and `withVize`
are aliases of `defineConfig` with the same single-object API.

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

Custom tasks use `vp run`. Built-in `vp check`, `vp lint`, and `vp fmt` do not
invoke these tasks. Built-in `vp build`, `vp dev`, and `vp pack` already use the
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
with Oxfmt. Set `lint.vize: false` or `fmt.vize: false` to return the corresponding
responsibility to Vite+. The integration option `conflicts: false` disables
automatic overlap handling.

The compiler replaces an existing `vite:vue` plugin, including nested and async
plugin lists, to avoid compiling a Vue file twice. Move that plugin's options to
`compiler`, or use `compiler: false` to retain it.

## Library declarations and maps

```ts
export default defineConfig({
  pack: {
    entry: ["src/index.ts"],
    outDir: "dist",
    format: ["esm"],
    vize: {
      dts: true,
      declarationMap: true,
      sourcemap: true,
    },
  },
});
```

Vue library bundling uses Vize's Rolldown integration. Providing `pack.vize`
enables native declaration generation by default; `dts: false` opts out.
Declarations are emitted after a successful bundle, and type errors fail the
pack command. Native declaration generation replaces tsdown's declaration pass
for that pack entry, so it understands Vue SFCs.

`declarationDir` overrides the output directory, otherwise `pack.outDir` is used.
`tsconfig` selects a TypeScript project. `declarationMap` overrides that project's
map setting without modifying authored tsconfig files. `sourcemap` controls
compiler and bundle maps. Each entry in a `pack` array can configure `vize`
independently; `pack.vize: false` leaves that entry with Vite+.

## Editor setup

Run `vp run editor:setup` to add the Vize and Vite Plus extension recommendations
and Vue-specific editor defaults. It preserves comments, existing recommendations,
explicit settings, and the formatter chosen for other languages.

See [Vite+ editor setup](./vite-plus-editor.md) for the recommended responsibilities
and how to share native configuration with the editor.
