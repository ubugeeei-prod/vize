# oxlint-plugin-vize

Oxlint JS plugin bridge for Vize Patina.

This package lets Oxlint execute Patina through Vize's native binding while still using Oxlint's JS plugin model and rule configuration.

> [!IMPORTANT]
> `oxlint-plugin-vize` is a terminal-first Vue SFC linting package.
> Until upstream Vue support in Oxlint matures, prefer `oxlint-vize -f stylish` for day-to-day output and treat machine-readable / full-fidelity original-SFC reporting as best-effort.

## Main Features

- Runs Vize Patina rules inside Oxlint as `vize/*` diagnostics, so Vue-specific findings can live beside Oxlint core rules in one command.
- Keeps Oxlint's existing rules and built-in `vue` plugin active. The bridge adds Vize rules; it does not replace `eqeqeq`, `no-console`, or your existing `vue/*` setup.
- Ships preset rule maps for JS/TS Oxlint configs: `configs.recommended`, `configs.essential`, `configs.ecosystem`, `configs.opinionated`, `configs.nuxt`, `configs.all`, and type-aware opt-in variants.
- Ships `createVizeLintConfig()` for the Vite+ `lint` block in `vite.config.ts`, which is the only Oxlint configuration `vp lint` and `vp check` read.
- Supports runtime settings through `settings.vize`, including `locale`, `preset`, and `helpLevel`.
- Provides the `oxlint-vize` CLI wrapper, which runs Oxlint with a scriptless-SFC workaround and rewrites temporary paths back to the original `.vue` files.
- Resolves Vize native bindings through platform-specific optional dependencies, so published installs do not need a separate `@vizejs/native` package.
- Caches file contents and native rule results for the lifetime of the Oxlint process, reducing duplicate work when several Vize rules are enabled for the same file.

## Performance

The bridge is optimized around Oxlint's per-rule execution model:

- The first enabled Patina rule on a file runs native linting for that rule only.
- If a second Patina rule is encountered on the same file, the bridge upgrades to one shared full-file Patina pass and reuses that result for the remaining Patina rules.
- Exact source revisions and rule results are cached for up to 128 recently used file/settings pairs; edits replace revision-local diagnostics and reporting state.

## Installation

`oxlint-plugin-vize` targets Node 22 and Node 24+ (`^22 || >= 24`). In this repository, Vite+ reads `.node-version` for you, so the usual setup is:

```bash
vp install
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add it with:

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` pulls the appropriate Vize native binding for the current platform through optional dependencies, so no separate `@vizejs/native` install is required for published builds.

## Usage

Which file you configure depends on which command you run.

| Command                 | Reads                                |
| ----------------------- | ------------------------------------ |
| `vp lint`, `vp check`   | the `lint` block in `vite.config.ts` |
| `oxlint`, `oxlint-vize` | `.oxlintrc.json` (or `-c <path>`)    |

> [!IMPORTANT]
> Vite+ never reads `.oxlintrc.json`. A `.oxlintrc.json` carrying `jsPlugins` and `vize/*` rules
> looks configured, but `vp lint` ignores the file, so Oxlint never sees a `vize/*` rule id and
> reports **zero** Vize diagnostics while exiting `0`. `vp lint --init` does not migrate an existing
> `.oxlintrc.json` either: it writes a fresh `lint` block and leaves the old file in place.
> If you use `vp lint`, configure the `lint` block.

### With `vp lint` (Vite+)

`createVizeLintConfig()` returns the whole `lint` block, so the `jsPlugins` entry cannot go missing:

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: createVizeLintConfig({
    preset: "essential",
    rules: {
      "no-console": "warn",
    },
    settings: {
      helpLevel: "short",
    },
  }),
});
```

`preset` drives both the emitted rule map and `settings.vize.preset`, so the two can never disagree.
That matters because the bridge silently drops any `vize/*` rule outside the active preset: listing
`vize/ecosystem/router-link-require-to` while the active preset is `general-recommended` reports
nothing at all. `createVizeLintConfig` throws for that case, and for unknown `vize/*` ids, instead of
leaving you with a config that looks enabled and reports nothing. Use `preset: "incremental"` when
you want only the rules you list, and `preset: "all"` for every bundle at once.

The emitted block always enables Oxlint's built-in `vue` plugin. Pass `plugins` to keep the rest of
your project's plugin list; they are merged with `vue`, never replaced, because narrowing the list
would silently drop everything those plugins report. A `create-vue` project keeps its generated set
like this:

```ts
export default defineConfig({
  lint: {
    ...createVizeLintConfig({
      plugins: ["eslint", "typescript", "unicorn", "oxc"],
    }),
    ignorePatterns: ["dist/**"],
  },
});
```

### With `oxlint` or `oxlint-vize`

Enable Oxlint's built-in `vue` plugin as well as this JS plugin:

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

This bridge only adds the `vize/*` rules. Oxlint's existing core rules and built-in plugin rules still run as configured, so checks like `eqeqeq`, `no-console`, or your existing `vue/*` setup continue to report normally.

If you want a lower-config JS/TS Oxlint setup, the package also exports preset rule maps:

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

`configs.recommended`, `configs.essential`, `configs.ecosystem`, `configs.opinionated`, `configs.nuxt`, and `configs.all` intentionally skip Vize's unstable type-aware rules for now. If you explicitly want those experimental rules too, use `configs.recommendedWithTypeAware`, `configs.ecosystemWithTypeAware`, `configs.opinionatedWithTypeAware`, or `createVizeRuleConfig({ includeTypeAware: true, preset: ... })`. Set `settings.vize.typeAware: true` to run the shared full-file Patina pass with Corsa enabled; explicitly configured `vize/type/*` rules also opt in when they are queried one by one.

You can pass Patina settings through `settings.vize`:

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "essential",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `preset` accepts `"general-recommended"`, `"essential"`, `"ecosystem"`, `"incremental"`, `"opinionated"`, or `"nuxt"`.
- `preset` defaults to `"general-recommended"`.
- Bundle presets keep out-of-bundle rules quiet even if they are still listed in `rules`.
- `"incremental"` skips bundle gating and runs only the Vize rules you explicitly configure in Oxlint.
- `"ecosystem"` enables Vize's Vue Router, Vue I18n, Pinia, Vue Test Utils, Nuxt, and Void Vue rules without taking on the full opinionated preset.
- `"opinionated"` is the preset that enables Vize's built-in script rules such as `vize/script/no-options-api`.
- Legacy aliases such as `"GeneralRecommended"`, `"Essential"`, `"Ecosystem"`, `"Incremental"`, `"Opinionated"`, `"Nuxt"`, and `"happy-path"` are still accepted for compatibility.
- `helpLevel` accepts `"full"`, `"short"`, or `"none"`. Short help keeps one actionable sentence and removes inline examples and trailing rationale.
- `helpLevel: "full"` only expands the Patina remediation text. It does not restore original-SFC formatter anchors or machine-readable range fidelity.
- `typeAware: true` enables Corsa-backed `vize/type/*` rules during shared Patina passes.
- `corsaPath` selects a Corsa or TypeScript 7 native executable for type-aware linting. Omit it to use Vize's normal resolver.
- `showHelp` is still accepted for backward compatibility, but `helpLevel` is the preferred setting.

For example, this keeps Oxlint focused on correctness-only Vize diagnostics while still allowing your existing Oxlint rules to run unchanged:

```json
{
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "vize/vue/require-v-for-key": "error",
    "vize/vue/require-scoped-style": "error"
  }
}
```

In that config, `vize/vue/require-v-for-key` can report, while `vize/vue/require-scoped-style` stays silent because it belongs to the broader `"general-recommended"` preset.

If you want to adopt Vize one rule at a time, use `"preset": "incremental"`. In that mode, preset membership no longer suppresses configured rules, so only the Vize rules you list under `rules` will run.

For day-to-day terminal runs, the recommended command today is:

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` is a thin wrapper around `oxlint`. Until upstream JS plugin coverage improves, it appends a temporary `<script setup>` block only for scriptless `.vue` files so Oxlint's JS plugin pipeline still invokes Vize, then rewrites reported paths back to the original files.
`stylish` is currently the most usable compromise for mixed Oxlint + Vize output because the Patina summary can inline the original SFC location even though Oxlint still anchors JS plugin diagnostics to the extracted script program.

## Limitations

- Raw `oxlint` still misses files without `<script>` or `<script setup>`. The temporary `oxlint-vize` wrapper works around this by generating a transient script block for scriptless `.vue` files before invoking `oxlint`.
- Oxlint JS plugins only accept ranges inside the extracted Vue script program. For template diagnostics, Vize now inlines the original SFC block and `line:column` into the summary, while the formatter anchor still points at the script block.
- Formatter parity is not there yet. `stylish` is recommended for human-readable terminal output, while `json` and other machine-readable outputs are best treated as debugging aids for original template/style positions.
- Oxlint core rules that need JavaScript bindings extracted from Vue templates, such as template-aware unused-variable checks, still depend on upstream work in [Oxc's Better Vue Support](https://github.com/oxc-project/oxc/issues/15761).
- Vize's own SFC diagnostics can run through the plugin, but precise original-SFC ranges across all Oxlint formatters depend on the JS plugin reporting work tracked in [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465).
- Type-aware Vize rules are experimental and excluded from the default exported configs. Opt into them explicitly with `configs.recommendedWithTypeAware`, `configs.opinionatedWithTypeAware`, or `createVizeRuleConfig({ includeTypeAware: true, preset: ... })`, and use `settings.vize.typeAware: true` when you want the shared full-file pass to run them eagerly.

## Current expectations

- This release is meant for terminal-first workflows.
- It is not yet a promise of precise original-SFC spans across every Oxlint formatter.
- Once Oxlint can preserve original Vue positions for JS plugins reliably, Vize can improve formatter parity and machine-readable reporting without relying on summary fallbacks.
