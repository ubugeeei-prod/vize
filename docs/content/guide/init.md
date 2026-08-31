---
title: Project Setup
---

# Project Setup With `vize init`

`vize init` selects, installs, and configures Vize in an existing project in one pass. It is
interactive by default and fully scriptable with flags.

```bash
vpx vize init
```

It configures five things, each of which you can take or leave:

| Feature          | What it wires up                                                   |
| ---------------- | ------------------------------------------------------------------ |
| oxlint plugin    | `oxlint-plugin-vize`, in the file your lint command actually reads |
| vite plugin      | `@vizejs/vite-plugin` in `vite.config.*`                           |
| nuxt module      | `@vizejs/nuxt` in `nuxt.config.*`                                  |
| fmt              | `vize.config.ts` formatter block and the `vize:fmt` scripts        |
| typecheck        | `vize.config.ts` type-checker block and the `vize:check` script    |
| editor extension | a `.vscode/extensions.json` recommendation                         |

The vite plugin and the nuxt module are the same choice: `init` detects which one your project needs.

## What It Detects

Detection runs first and is printed before you are asked anything, so you can catch a wrong
directory or a missing lockfile before any file is touched:

```
[vize init] detected in /path/to/project:
  framework:       Vite+ (vite.config.ts)
  package manager: pnpm
  language:        TypeScript (tsconfig.json)
  lint command:    vp lint
  vize config:     none
  oxlint config:   none
```

| Signal          | How it is decided                                                                                                                                                |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Framework       | `nuxt.config.{ts,mts,js,mjs}` or a `nuxt` dependency means Nuxt; otherwise a `vite.config.*` means Vite; otherwise neither                                       |
| Both present    | Nuxt wins — a Nuxt project owns its own Vite instance, so the module is the supported integration point. Override with `--vite`.                                 |
| Neither present | The bundler feature is offered as unavailable and the other four still apply. `init` does not invent a build setup for you.                                      |
| Package manager | The lockfile first (`pnpm-lock.yaml`, `bun.lockb`/`bun.lock`, `yarn.lock`, `package-lock.json`), then the `packageManager` field. A Vite+ project uses `vp add`. |
| Language        | `tsconfig.json` and a `typescript` dependency. Typecheck needs a `tsconfig.json`.                                                                                |
| Lint command    | `vp lint` when Vite+ is in use, otherwise the `oxlint` binary. **This decides which file the Oxlint configuration goes in.**                                     |
| Already done    | An existing Vize config, an existing Oxlint config, an existing `lint` block, plugins already wired, and installed dependencies                                  |

## Which Lint File It Writes

This is the part worth understanding, because getting it wrong fails silently.

| Command                 | Reads                                 |
| ----------------------- | ------------------------------------- |
| `vp lint`, `vp check`   | the `lint` block in `vite.config.ts`  |
| `oxlint`, `oxlint-vize` | `.oxlintrc.json` / `oxlint.config.ts` |

Vite+ never reads `.oxlintrc.json`. A `.oxlintrc.json` carrying `jsPlugins` and `vize/*` rules looks
configured, but `vp lint` ignores it, Oxlint never sees a `vize/*` rule id, and the run reports
**zero** Vize diagnostics while exiting `0`. See the [Oxlint Plugin guide](./oxlint.md) for the full
picture.

`init` therefore writes the file your lint command reads, not the one that is easiest to write:

| Your project                                    | What `init` writes                                                       |
| ----------------------------------------------- | ------------------------------------------------------------------------ |
| uses Vite+                                      | the `lint` block in `vite.config.*`, built with `createVizeLintConfig()` |
| does not use Vite+                              | `oxlint.config.ts`                                                       |
| uses Vite+ **and** runs `oxlint` directly       | both, generated from the same preset so they cannot drift apart          |
| uses Vite+ but its Vite config cannot be edited | nothing — see [When Init Refuses To Edit](#when-init-refuses-to-edit)    |

The `lint` block is emitted through `createVizeLintConfig()`, which returns the whole block rather
than fragments, so the `jsPlugins` entry that loads the bridge cannot go missing:

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: createVizeLintConfig({
    preset: "general-recommended",
    settings: {
      helpLevel: "short",
    },
  }),
  plugins: [vize()],
});
```

For a project without Vite+, the equivalent `oxlint.config.ts`:

```ts
import { defineConfig } from "oxlint";
import { configs } from "oxlint-plugin-vize";

export default defineConfig({
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      preset: "general-recommended",
      helpLevel: "short",
    },
  },
  rules: configs.recommended,
});
```

> [!NOTE]
> Oxlint auto-discovers `.oxlintrc.json`, `.oxlintrc.jsonc`, and `oxlint.config.ts`. It does **not**
> discover `oxlint.config.mjs`, `.js`, `.cjs`, `.mts`, or `.cts`. If your project holds one of those,
> `init` reports it as present but unread rather than treating it as configuration.

## Non-Interactive Use

`init` refuses to prompt when stdin is not a TTY, so it fails fast in CI instead of hanging. Pass
`--yes` together with the features you want:

```bash
vpx vize init --yes --lint --vite --fmt --typecheck --editor
```

| Option                           | Effect                                              |
| -------------------------------- | --------------------------------------------------- |
| `-y`, `--yes`                    | Accept the selection without prompting              |
| `--lint` / `--no-lint`           | The oxlint plugin                                   |
| `--vite`                         | Force the Vite plugin, even if Nuxt was detected    |
| `--nuxt`                         | Force the Nuxt module                               |
| `--bundler` / `--no-bundler`     | The vite plugin or nuxt module, auto-detected       |
| `--fmt` / `--no-fmt`             | `vize fmt`                                          |
| `--typecheck` / `--no-typecheck` | `vize check`, which needs a `tsconfig.json`         |
| `--editor` / `--no-editor`       | The `.vscode/extensions.json` recommendation        |
| `--dry-run`                      | Print the full plan and write nothing               |
| `--no-install`                   | Write configuration without installing dependencies |
| `--package-manager <PM>`         | One of `pnpm`, `npm`, `yarn`, `bun`, `vp`           |

`--dry-run` prints exactly what a real run would do:

```
[vize init] plan:
  lint      configured Vite+ detected, so `vp lint` reads the `lint` block in vite.config.ts and never reads .oxlintrc.json; writes vite.config.ts
  bundler   configured adds vize() to vite.config.ts
  fmt       configured writes vize.config.ts
  typecheck configured writes vize.config.ts
  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize
[vize init] would create vize.config.ts
[vize init] would create .vscode/extensions.json
[vize init] would update vite.config.ts
[vize init] would update package.json
[vize init] would add scripts: vize:lint, vize:fmt, vize:fmt:fix, vize:check
[vize init] would run: vp add -D @vizejs/vite-plugin oxlint oxlint-plugin-vize vize
```

## Running It Twice

`init` is idempotent. A second run adds no duplicate plugin entry, script, or dependency, and leaves
every file byte-identical:

```
[vize init] plan:
  lint      unchanged  Vite+ detected, so `vp lint` reads the `lint` block in vite.config.ts and never reads .oxlintrc.json
  bundler   unchanged  vite.config.ts already uses @vizejs/vite-plugin
  fmt       unchanged  vize.config.ts already exists and was left unchanged
  typecheck unchanged  vize.config.ts already exists and was left unchanged
  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize
[vize init] nothing to do; the project is already configured
```

Already-configured features stay ticked in the prompt and are labelled as such, so re-running to add
one more feature does not mean re-selecting everything.

## When Init Refuses To Edit

`init` never guesses at a build config. If it cannot make an edit safely it writes nothing, reports
the feature as **not configured**, prints the exact snippet to paste, and exits non-zero.

This matters most for lint. If your Vite config already has a `lint` block, `init` will not merge
into it, because merging risks dropping settings:

```
[vize init] lint: NOT configured — vp lint reads the `lint` block in the Vite config, but
vite.config.ts already has a `lint` block and merging into it would risk dropping settings, so
spread createVizeLintConfig() into it by hand

    import { createVizeLintConfig } from "oxlint-plugin-vize";

    export default defineConfig({
      lint: {
        ...createVizeLintConfig({
          preset: "general-recommended",
          settings: {
            helpLevel: "short",
          },
        }),
        // keep your existing lint keys here
      },
    });
```

Note what it does **not** do: fall back to writing `oxlint.config.ts`. That would leave `vp lint`
reading your untouched `lint` block and reporting nothing, which is worse than being unconfigured.
A project that fails loudly is fixable; one that is quietly silent is not.

The same rule applies to the other edits. `init` will decline when there are several Vite configs,
when the config is not a single plain `defineConfig({ ... })` call, when `plugins` is not an array
literal, or when `.vscode/extensions.json` is not a plain JSON object.

## Editor Support

`init` writes a recommendation rather than installing anything:

```json
{
  "recommendations": ["ubugeeei.vize"]
}
```

This is checked in, applies to everyone on the project, and changes nothing on the machine that ran
`init`. Vize also ships integrations for editors other than VS Code, which `init` lists on success:

| Editor  | Integration        |
| ------- | ------------------ |
| VS Code | `ubugeeei.vize`    |
| Zed     | `editors/zed`      |
| Neovim  | `editors/nvim`     |
| Vim     | `editors/vim`      |
| Helix   | `editors/helix`    |
| Emacs   | `editors/emacs`    |

See [VS Code and Other Editors](../integrations/vscode.md) for per-editor instructions.

## Relationship To `vize setup`

`vize setup` is the older, all-or-nothing command for a Vite or Vite+
project. It still works and is unchanged. `init` covers the same ground plus feature selection, Nuxt,
package-manager detection, a dry run, and a non-interactive mode, over the same planning layer. Prefer
`vize init` for new projects.

## Next Steps

- [Static Analysis](./static-analysis.md) for the lint and type-checking model
- [Oxlint Plugin](./oxlint.md) for preset and settings reference
- [Configuration](./configuration.md) for everything `vize.config.ts` accepts
- [CLI](./cli.md) for the rest of the commands
