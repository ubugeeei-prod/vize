# @vizejs/oxlint-plugin-patina

Oxlint JS Plugin bridge for Vize Patina.

This package lets Oxlint execute Patina through the native `@vizejs/native` binding while still using Oxlint's JS plugin model and rule configuration.

## Performance

The bridge is optimized around Oxlint's per-rule execution model:

- The first enabled Patina rule on a file runs native linting for that rule only.
- If a second Patina rule is encountered on the same file, the bridge upgrades to one shared full-file Patina pass and reuses that result for the remaining Patina rules.
- File contents and rule results are cached per file and locale for the lifetime of the Oxlint process.

## Installation

```bash
pnpm add -D oxlint @vizejs/oxlint-plugin-patina
```

## Usage

Enable Oxlint's built-in `vue` plugin as well as this JS plugin:

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["@vizejs/oxlint-plugin-patina"],
  "rules": {
    "patina/vue/require-v-for-key": "error",
    "patina/vue/no-v-html": "warn"
  }
}
```

You can pass Patina settings through `settings.patina`:

```json
{
  "settings": {
    "patina": {
      "locale": "ja"
    }
  }
}
```

## Current limitations

- Oxlint JS plugins currently rely on the extracted Vue script program. Files without `<script>` or `<script setup>` do not invoke the plugin yet.
- Oxlint does not expose custom Vue template/style ranges to JS plugins. When a Patina diagnostic cannot be mapped onto the extracted script range, the plugin prefixes the true `line/column` in the message and anchors the report to the current script program.
