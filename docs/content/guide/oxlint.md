---
title: Oxlint Plugin
---

# Oxlint Plugin

`oxlint-plugin-vize` lets Oxlint execute Vize Patina diagnostics through Oxlint's JS plugin system. This is useful when you want Oxlint's Rust-native JavaScript and TypeScript rules together with Vize's Vue-specific diagnostics in a single run.

> [!IMPORTANT]
> As of March 21, 2026, `oxlint-plugin-vize` is not yet on npm. The first public release is planned as an alpha.
> Until [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465) lands, prefer `oxlint-vize -f stylish` for human-readable terminal runs and treat machine-readable / full-fidelity original-SFC reporting as best-effort.

## Installation

Once the alpha release is published:

```bash
pnpm add -D oxlint oxlint-plugin-vize@alpha
```

`oxlint-plugin-vize` installs the matching Vize native binding for the current platform through optional dependencies, so no separate `@vizejs/native` install is required for consumers.

## Basic Usage

Enable Oxlint's built-in `vue` plugin and load the Vize bridge as a JS plugin:

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

This keeps Oxlint's existing rules running as-is, including core checks like `eqeqeq` and `no-console`, while adding Vize's Vue diagnostics under the `vize/vue/*` namespace. The bridge does not replace or suppress your other Oxlint rules.

If you are willing to use a JS/TS Oxlint config, `oxlint-plugin-vize` also exports preset rule maps so you do not need to list dozens of `vize/*` rules by hand:

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
    },
  },
  rules: configs.opinionated,
};
```

`configs.recommended`, `configs.essential`, `configs.opinionated`, `configs.nuxt`, and `configs.all` intentionally skip Vize's unstable type-aware rules during the alpha period. If you want to opt into them, use `configs.recommendedWithTypeAware`, `configs.opinionatedWithTypeAware`, or `createVizeRuleConfig({ includeTypeAware: true, preset: ... })`.

For day-to-day terminal usage, the recommended command today is:

```bash
pnpm exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` is a thin wrapper around `oxlint`. During the alpha period, it appends a temporary `<script setup>` block only for scriptless `.vue` files so Oxlint's JS plugin pipeline still invokes Vize, then rewrites the reported path back to the original file. This workaround is intended to be removed once upstream coverage improves.
`stylish` is currently the best compromise for mixed Oxlint + Vize output because Vize can inline the original SFC location into the message body even though Oxlint still anchors JS plugin diagnostics to the extracted script program.

## Settings

Patina settings are passed through `settings.vize`:

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "essential",
      "helpLevel": "short"
    }
  }
}
```

- `locale` controls the diagnostic language.
- `preset` accepts `"general-recommended"`, `"essential"`, `"incremental"`, `"opinionated"`, or `"nuxt"`.
- `preset` defaults to `"general-recommended"`.
- Bundle presets keep out-of-bundle rules quiet even if they are still listed in `rules`.
- `"incremental"` skips bundle gating and runs only the Vize rules you explicitly configure in Oxlint.
- `"opinionated"` is the preset that enables built-in script rules such as `vize/script/no-options-api`.
- Legacy aliases such as `"GeneralRecommended"`, `"Essential"`, `"Incremental"`, `"Opinionated"`, `"Nuxt"`, and `"happy-path"` are still accepted for compatibility.
- `helpLevel` accepts `"full"`, `"short"`, or `"none"`.
- `helpLevel: "full"` only expands the Patina remediation text. It does not restore original-SFC formatter anchors or machine-readable range fidelity.
- `showHelp` is still accepted for backward compatibility, but `helpLevel` is the preferred setting.

For compatibility with older configs, `settings.patina` is still accepted, but `settings.vize` is the canonical key.

If you want the plugin to behave like a minimal correctness pass, use `"preset": "essential"`. If you want the stricter Vapor-oriented script checks too, switch to `"preset": "opinionated"`.

If you want to roll Vize out one rule at a time, use `"preset": "incremental"`. In that mode, the plugin does not inherit any preset bundle; it only runs the Vize rules you explicitly turned on in Oxlint.

## How It Works

- The first enabled Vize rule on a file runs a native Patina pass for that rule.
- If a second Vize rule is enabled for the same file, the bridge upgrades to one shared full-file Patina pass and reuses the result for the remaining rules.
- Native bindings are resolved by platform package first, then by the workspace `@vizejs/native` package during local development.

## Current Limitations

- Raw `oxlint` still misses files without `<script>` or `<script setup>`. The temporary `oxlint-vize` wrapper works around this by generating a transient script block for scriptless `.vue` files before invoking `oxlint`.
- Oxlint JS plugins only accept ranges inside the extracted Vue script program. For template diagnostics, Vize inlines the original SFC block and `line:column` into the summary, while the fallback formatter anchor still points at the script block.
- Because of [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465), formatter parity is not there yet. `stylish` is usable for humans, but `json` and other machine-readable outputs should be treated as debugging aids rather than a source of truth for original template/style positions.

## Alpha Scope

- The planned alpha release is for human-in-the-loop terminal linting first, especially `oxlint-vize -f stylish`.
- The alpha is not yet a promise of precise original-SFC spans across every Oxlint formatter.
- Once Oxlint's JS plugin reporting can preserve original Vue positions reliably, Vize can improve formatter parity, CI parser friendliness, and deeper editor/reporting integrations without relying on summary fallbacks.

## Local Development in This Repository

If you are working inside the Vize repository itself:

```bash
vp install
vp run --filter './npm/vize-native' build
vp run --filter './npm/oxlint-plugin-vize' build
```

The package build uses `vp pack`, targets Node 24+, and follows the repository's `.node-version`.
