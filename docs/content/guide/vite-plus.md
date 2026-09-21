---
title: Vite+
---

# Vite+

`withVize` connects the Vize compiler, native typechecker, native linter, and formatter
to Vite+ from one config. Install `@vizejs/vite-plugin` and your preferred version of
`vite-plus` in the project. Vite+ is an optional peer dependency: Vize does not pin,
download, or replace it. Plain Vite users can keep the ordinary plugin entry.

```ts
// vite.config.ts
import { withVize } from "@vizejs/vite-plugin/vite-plus";

export default withVize();
```

No extra plugin registration or Vize CLI scripts are needed. With no argument,
the helper reads an existing `vize.config.*` when present. You can instead pass
the shared configuration directly, including an async config function or scoped entries:

```ts
import { withVize } from "@vizejs/vite-plugin/vite-plus";

export default withVize({
  compiler: { sourceMap: true },
  linter: { preset: "essential", rules: { "vue/no-v-html": "error" } },
  formatter: { singleQuote: true },
}).vp({
  server: { port: 3000 },
  lint: { rules: { "no-debugger": "error" } },
  fmt: { ignorePatterns: ["generated/**"] },
});
```

`.vp(...)` accepts the installed Vite+ `defineConfig` input, including promises and
async config functions. Its types come from Vite+; Vize does not maintain a copy.
Shared Vize config functions receive the native command (`check`, `lint`, or `fmt`)
when tasks run, with production mode. Build and dev use Vite's config environment.

## Tasks

| Command | Work |
| --- | --- |
| `vp run check` | Native typecheck, native lint, Oxlint, native Vue format check, Oxfmt check |
| `vp run check -- --fix` | Typecheck, fix lint, and write formatting |
| `vp run lint` | Native lint and Oxlint; both run even if one reports errors |
| `vp run lint:fix` | Fix through both lint engines |
| `vp run fmt` | Vize writes Vue formatting; Oxfmt writes other files |
| `vp run fmt:check` | Check both formatters without writing |
| `vp run build` | Vite+ build using the Vize compiler plugin |
| `vp run dev`, `preview`, `test` | The corresponding installed Vite+ commands |

Pass paths after `--` to check, lint, and format tasks. `fmt` also accepts `--check`
or `--write`. Configure other options in the shared config or `.vp(...)`.
Tasks report failure if either tool fails. They are uncached by default so fixes
and unsaved configuration changes cannot be hidden by a task cache.

Vite+ currently exposes custom tasks through `vp run`. Its built-in `vp check`,
`vp lint`, and `vp fmt` do not run these custom tasks. `vp build` and `vp dev`
already use the installed compiler plugin.

Existing package scripts are preserved. If `package.json` has a `check` script,
the generated task is called `vize:check`; invoke `vp run vize:check`. The same
rule applies to the other generated task names. Explicit `run.tasks` definitions
take precedence. Conflicting explicit renames produce an error with a rename hint.

## Tool ownership and overrides

Lint runs Vize's native linter alongside Oxlint, including Oxlint's JS/TS diagnostics
inside Vue files. It does not install or run `oxlint-plugin-vize`. Overlapping Vue
rules are disabled in Oxlint by default, using the installed Vite+ rule catalog to
avoid referencing rules unavailable in that version. Explicit `.vp({ lint: { rules } })`
settings and overrides can re-enable a rule.

Formatting assigns `**/*.vue` to Vize and excludes it from Oxfmt. Configure Vue
formatting in `withVize({ formatter: ... })`; `.vp({ fmt: ... })` controls Oxfmt.
Use shared Vize `ignores`/scoped entries for native exclusions, and the corresponding
Vite+ tool's `ignorePatterns` for its exclusions.

The optional second argument controls integration:

```ts
export default withVize(
  { linter: { preset: "essential" } },
  {
    tasks: { check: "verify", preview: false },
    plugin: { sourceMap: true },
  },
).vp({ run: { tasks: { deploy: "your-deploy-command" } } });
```

- `tasks: false` disables generated tasks. Individual names can be renamed or disabled.
- `plugin: false` keeps an existing compiler; otherwise pass ordinary Vize plugin options.
- `check: false`, `lint: false`, or `fmt: false` disables the corresponding native task
  step. Oxlint/Oxfmt regain ownership when their native counterpart is disabled.
- `conflicts: false` disables the overlap defaults, for projects that manage tool
  ownership themselves.

Direct Vize CLI commands and editors still discover `vize.config.*`. To share the
same config with those consumers, keep it in that file and use `withVize()` or
import the config into `withVize(config)`.

When compilation is enabled, the helper replaces an existing `vite:vue` plugin
from `@vitejs/plugin-vue`, including nested and async plugin lists, to prevent
double compilation. Move custom compiler options to the `plugin` option above.
Use `plugin: false` to keep an existing compiler in charge.
