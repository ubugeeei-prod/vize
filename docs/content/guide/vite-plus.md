---
title: Vite+
---

# Vite+

Use Vite+'s own `defineConfig` for Vite+, the Vize compiler, native linter,
formatter, and library declarations. Keeping the outer call owned by Vite+
lets `vp run` read literal tasks without bundling or evaluating the config.
Type-check with the content mapper.
Install `@vizejs/vite-plugin` and
your preferred compatible `vite-plus` version in the project:

```bash
vp install -D @vizejs/vite-plugin
```

Keep the integration settings in **`vite.config.ts`**; no separate Vize config is
required for these tasks. [Migrate an existing project](./migration.md) with literal
import/plugin diffs, or follow [Getting Started](../getting-started.md).

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { vitePlusConfig } from "@vizejs/vite-plugin/vite-plus";

const vize = await vitePlusConfig({ vue: {} });

export default defineConfig({
  ...vize,
  run: {
    tasks: {
      "editor:setup": { command: "vize-vp-task editor:setup", cache: false },
      check: { command: "vize-vp-task check", cache: false },
      typecheck: { command: "vize-vp-task typecheck", cache: false },
      lint: { command: "vize-vp-task lint", cache: false },
      "lint:fix": { command: "vize-vp-task lint:fix", cache: false },
      fmt: { command: "vize-vp-task fmt", cache: false },
      "fmt:check": { command: "vize-vp-task fmt:check", cache: false },
      build: { command: "vize-vp-task build", cache: false },
      pack: { command: "vize-vp-task pack", cache: false },
      dev: { command: "vize-vp-task dev", cache: false },
      preview: { command: "vize-vp-task preview", cache: false },
      test: { command: "vize-vp-task test", cache: false },
    },
  },
});
```

> [!IMPORTANT]
> Static task extraction requires Vite+ 1.5.0 or later. Keep every task literal
> inside `run.tasks`; spreading a generated task object makes Vite+ evaluate the
> config again.

Vite+ is an optional peer dependency. Vize uses the project's installed version
and its configuration types; it never downloads or replaces Vite+.
Ordinary Vite projects can keep the usual plugin entry point.

## Configuration

```ts
import { defineConfig } from "vite-plus";
import { vitePlusConfig } from "@vizejs/vite-plugin/vite-plus";
import { defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";

const vize = await vitePlusConfig({
  server: { port: 3000 },
  lint: {
    rules: { "no-debugger": "error" },
    ignorePatterns: ["dist/**"],
  },
  fmt: { ignorePatterns: ["dist/**"] },
  vue: {
    lint: defineVizeLintConfig(...flatConfigs.recommended, ...flatConfigs.ecosystem, {
      ignorePatterns: ["dist/**"],
      rules: { "no-console": "warn" },
    }),
    fmt: { singleQuote: true },
    entries: [
      {
        files: ["src/admin/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});

export default defineConfig({ ...vize });
```

Keep the root object as an ordinary Vite+ config and put shared Vue settings in
its `vue` key. `vitePlusConfig()` composes the Vue flat lint layer with that root
configuration, so the root
`no-debugger` policy remains active alongside `flatConfigs.recommended`,
`flatConfigs.ecosystem`, and the Vue-specific `no-console` rule. Use `compile`,
`typeCheck`, `lint`, and `fmt`. `lint` is the host Oxlint block; its
`settings.vize` and `vize/*` rules also configure the native linter. Keep
`entries[].linter` for file-scoped native overrides. The older `compiler`,
`typeChecker`, `linter`, and `formatter` names remain compatibility aliases, but
do not specify an alias together with its replacement. See [Shared
Vue configuration](./vue-configuration.md) for paths and scoped entries.

| Section                               | Owner                                                                                                              |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `vue.compile` (`compiler` alias)      | Native compiler settings shared with the CLI and editor                                                            |
| `vue.typeCheck` (`typeChecker` alias) | Native checker settings; TypeScript defaults come from `tsconfig.json`                                             |
| `vue.lint` (`linter` alias)           | Host Oxlint settings; `settings.vize`, `vize/*` rules, and overrides project into native Vue lint                  |
| `vue.fmt` (`formatter` alias)         | Native Vue formatting options                                                                                      |
| `vue.languageServer`                  | Shared language-server features                                                                                    |
| `compiler`                            | Vite+ compiler plugin controls; `false` keeps the existing Vue compiler when `vue.compiler` is absent              |
| `typecheck`                           | Vite+ native-check task controls; `false` disables the task unless `vue.typeChecker.enabled` explicitly enables it |
| `lint.vue`                            | Native lint task options, including `typecheck`, `locale` and `helpLevel`                                          |
| Other `lint` fields                   | The installed Vite+ / Oxlint configuration; `ignorePatterns` also excludes native lint inputs                      |
| `fmt.vue`                             | Native formatter task options; `false` returns formatting to Oxfmt when `vue.formatter` is absent                  |
| Other `fmt` fields                    | The installed Vite+ / Oxfmt configuration; `ignorePatterns` also excludes native Vue formatting                    |
| `pack.vue`                            | Native declaration and map options                                                                                 |
| Other `pack` fields                   | The installed Vite+ / tsdown configuration                                                                         |

Canonical `vue` settings take precedence over the corresponding task convenience
options. Supplying `vue.lint` makes the host Oxlint block own the lint pass and
disables the duplicate native lint task. Keep TypeScript strictness and template
options in the existing TypeScript project:

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
additions. The static integration accepts a literal object.

### Dynamic configuration compatibility

The original wrapper remains available for environment-dependent configuration
functions and Vite+ versions before 1.5. It evaluates the config before task lookup:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: { sourceMap: true },
  formatter: { singleQuote: true },
});
```

This preserves existing Vite+ options, plugins and hooks and injects the Vue
compiler once. There is no custom config `extends`, and dedicated `vize.config.*`
files and top-level `vize:` settings are retired. Standard TypeScript `extends`
and `references` in `tsconfig.json` remain supported.

The standalone `withVue` helper and the `withVize` compatibility name remain
supported, but literal task-heavy projects should use `vitePlusConfig()` and
Vite+'s outer `defineConfig`.

## Tasks

| Command                  | Work                                                               |
| ------------------------ | ------------------------------------------------------------------ |
| `vp run check`           | Packaged typecheck, native lint, Oxlint, and both formatter checks |
| `vp run check -- --fix`  | Packaged typecheck, fix lint, and write formatting                 |
| `vp run typecheck`       | Packaged checker only                                              |
| `vp run lint`            | Native lint and Oxlint, plus optional native typecheck             |
| `vp run lint:fix`        | Fix through both lint engines                                      |
| `vp run fmt`             | Vize writes Vue formatting; Oxfmt writes other files               |
| `vp run fmt:check`       | Check both formatters without writing                              |
| `vp run build`, `dev`    | Vite+ with the Vize compiler                                       |
| `vp run pack`            | Vite+ library bundling and configured native declarations          |
| `vp run preview`, `test` | The corresponding installed Vite+ commands                         |
| `vp run editor:setup`    | Recommend extensions and Vue editor defaults                       |

The recommended typecheck is the [content mapper](./content-mapper.md). TypeScript 7.1 runs `tsc --runExternalCode --noEmit`. tsrs is a drop-in of the same `tsconfig`: `tsrs --runExternalCode --noEmit`. `vp run typecheck` is the packaged checker, and `vp run check` includes it.

Pass paths after `--` to check, lint, and format tasks. Formatting also accepts
`--check` or `--write`. Put tool options in the corresponding config section.
Checks continue after diagnostics so one failing tool does not hide the others.
The literal tasks above are uncached by default. Read-only `lint`, `fmt:check`,
`check`, and `typecheck` tasks may use Vite+ caching explicitly:

```ts
const check = {
  command: "vize-vp-task check",
  cache: {
    input: [{ auto: true }, "!node_modules/.vize/**", "!**/.vize/**"],
    output: [],
  },
};
```

Vite+ automatically tracks source files, configuration imports, dependencies, and
directory entries. Generated `node_modules/.vize/**` and `**/.vize/**` files are
excluded from cache inputs, and checks never restore output files. Source-writing
tasks such as `fmt` and `lint:fix` remain uncached. Passing `--fix` to a cached
check still runs on changed inputs, including reintroducing the same unformatted
source after a previous fix.

> [!WARNING]
> Custom tasks use `vp run`. Built-in `vp check`, `vp lint`, and `vp fmt` do not
> invoke these tasks.

Built-in `vp build`, `vp dev`, and `vp pack` already use the
configured compiler or pack hooks.

Existing package scripts are preserved. If a literal task name already exists as
a package script, rename its `run.tasks` key (for example, `vize:check`) or omit
that task. Keep the task object literal after editing it so static extraction
remains available.

The compatibility wrapper still supports generated task renaming for dynamic
configs:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig(
  { run: { tasks: { deploy: "your-deploy-command" } } },
  { tasks: { check: "verify", preview: false } },
).withVue({});
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

`lint.ignorePatterns` applies to both linters, so one list excludes generated or
vendored Vue, JavaScript, and TypeScript inputs from both native lint and Oxlint.
`fmt.ignorePatterns` likewise applies to both formatters, so generated or vendored Vue files
stay untouched. An entry that excludes every Vue file (`**/*.vue` or `*.vue`) only keeps
`.vue` away from Oxfmt, as the integration does by default, so `conflicts: false` projects
can add it themselves; `vize fmt` still formats them. To use `lint.vue: false` or `fmt.vue: false`, omit the corresponding
shared native settings that would explicitly enable that tool. `lint.vue.locale` (`en`, `ja`, or `zh`) and `helpLevel` (`full`,
`short`, or `none`) control native diagnostics without passing unsupported flags
to Oxlint.

The compiler replaces an existing `vite:vue` plugin, including nested and async
plugin lists, to avoid compiling a Vue file twice. Move native options to `vue.compiler` and plugin-only controls to `compiler`.
To retain the original compiler, omit `vue.compiler` and use `compiler: false`.

## Library declarations and maps

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
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
}).withVue({});
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
