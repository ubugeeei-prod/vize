# oxlint-plugin-vize

Oxlint JS plugin bridge for Vize Patina.

This package lets Oxlint execute Patina through Vize's native binding while still using Oxlint's JS plugin model and rule configuration.

> [!IMPORTANT]
> As of March 21, 2026, `oxlint-plugin-vize` is not yet on npm. The first public release is planned as an alpha.
> Until [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465) lands, prefer `oxlint-vize -f stylish` for terminal workflows and treat machine-readable / full-fidelity original-SFC reporting as best-effort.

## Performance

The bridge is optimized around Oxlint's per-rule execution model:

- The first enabled Patina rule on a file runs native linting for that rule only.
- If a second Patina rule is encountered on the same file, the bridge upgrades to one shared full-file Patina pass and reuses that result for the remaining Patina rules.
- File contents and rule results are cached per file and locale for the lifetime of the Oxlint process.

## Installation

`oxlint-plugin-vize` targets Node 24+. In this repository, Vite+ reads `.node-version` for you, so the usual setup is:

```bash
vp install
vp run --filter './npm/vize-native' build
vp run --filter './npm/oxlint-plugin-vize' build
```

Once the alpha release is published, install it with:

```bash
pnpm add -D oxlint oxlint-plugin-vize@alpha
```

`oxlint-plugin-vize` pulls the appropriate Vize native binding for the current platform through optional dependencies, so no separate `@vizejs/native` install is required for published builds.

## Usage

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

If you want a lower-config JS/TS Oxlint setup during the alpha period, the package also exports preset rule maps:

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

`configs.recommended`, `configs.essential`, `configs.opinionated`, `configs.nuxt`, and `configs.all` intentionally skip Vize's unstable type-aware rules for now. If you explicitly want those alpha-stage rules too, use `configs.recommendedWithTypeAware`, `configs.opinionatedWithTypeAware`, or `createVizeRuleConfig({ includeTypeAware: true, preset: ... })`.

You can pass Patina settings through `settings.vize`:

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

- `preset` accepts `"general-recommended"`, `"essential"`, `"incremental"`, `"opinionated"`, or `"nuxt"`.
- `preset` defaults to `"general-recommended"`.
- Bundle presets keep out-of-bundle rules quiet even if they are still listed in `rules`.
- `"incremental"` skips bundle gating and runs only the Vize rules you explicitly configure in Oxlint.
- `"opinionated"` is the preset that enables Vize's built-in script rules such as `vize/script/no-options-api`.
- Legacy aliases such as `"GeneralRecommended"`, `"Essential"`, `"Incremental"`, `"Opinionated"`, `"Nuxt"`, and `"happy-path"` are still accepted for compatibility.
- `helpLevel` accepts `"full"`, `"short"`, or `"none"`.
- `helpLevel: "full"` only expands the Patina remediation text. It does not restore original-SFC formatter anchors or machine-readable range fidelity.
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
pnpm exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` is a thin wrapper around `oxlint`. During the alpha period, it appends a temporary `<script setup>` block only for scriptless `.vue` files so Oxlint's JS plugin pipeline still invokes Vize, then rewrites reported paths back to the original files. This workaround is intended to be removed once upstream JS plugin coverage improves.
`stylish` is currently the most usable compromise for mixed Oxlint + Vize output because the Patina summary can inline the original SFC location even though Oxlint still anchors JS plugin diagnostics to the extracted script program.

## Current limitations

- Raw `oxlint` still misses files without `<script>` or `<script setup>`. The temporary `oxlint-vize` wrapper works around this by generating a transient script block for scriptless `.vue` files before invoking `oxlint`.
- Oxlint JS plugins only accept ranges inside the extracted Vue script program. For template diagnostics, Vize now inlines the original SFC block and `line:column` into the summary, while the formatter anchor still points at the script block.
- Because of [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465), formatter parity is not there yet. `stylish` is recommended for human-readable terminal output, while `json` and other machine-readable outputs are best treated as debugging aids for now.

## Alpha expectations

- The planned alpha release is meant for terminal-first workflows.
- The alpha is not yet a promise of precise original-SFC spans across every Oxlint formatter.
- Once Oxlint can preserve original Vue positions for JS plugins reliably, Vize can improve formatter parity and machine-readable reporting without relying on summary fallbacks.
