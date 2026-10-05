## Area

`@vizejs/nuxt` lint config generation (`.nuxt/oxlint.config.json`, read by the dev `checker`)

## Version

`@vizejs/nuxt` / `oxlint-plugin-vize` 0.432.0; Oxlint 1.75.0 and 1.86.0 (both behave the same)

## What happens

To fix #7251, 0.432.0 rebases every glob onto the config's directory (`rebaseGlob()` in
`dist/generation-*.mjs`: `path.relative(configDir, path.resolve(rootDir, glob))`). Because the
config lives in `.nuxt/`, every glob now starts with `../`. Oxlint does not accept that:

1. `ignorePatterns`: Oxlint refuses to load the file, so the dev checker cannot lint anything.
   In a Nuxt 4 app using the module's lint integration, the generated file contains
   `"ignorePatterns": ["../**/dist", "../**/node_modules", "../**/.nuxt", "../**/.output", "../**/.vercel", "../**/.netlify", "../**/public"]`,
   and running the checker's command on it (`oxlint-vize --config .nuxt/oxlint.config.json …`)
   fails with the error below.
2. `overrides[].files`: a `../…` glob matches nothing, so the page/layout overrides, and items
   added through `vize:lint:config:addons`, still never apply. (Before 0.432.0 a project could
   work around #7251 by writing `**/app/pages/**/*.vue`; the rebase turns that into
   `../**/app/pages/**/*.vue`, which no longer matches either.)

## Minimal reproduction

The renderer the module uses, called directly so the output is exactly what 0.432.0 writes:

`render.mjs`

```js
import fs from "node:fs";
import path from "node:path";
// renderNuxtOxlintConfig from @vizejs/nuxt 0.432.0 (internal chunk, imported by file path:
// node_modules/@vizejs/nuxt/dist/generation-CXADZreI.mjs, exported there as `o`)
import { o as renderNuxtOxlintConfig } from "./node_modules/@vizejs/nuxt/dist/generation-CXADZreI.mjs";

const rootDir = process.cwd();
const configDir = path.join(rootDir, ".nuxt");
const items = [
  { name: "ignores", ignores: ["**/dist"] },
  { name: "all vue", files: ["**/*.vue"], rules: { "vue/no-inline-style": "warn" } },
  { name: "pages", files: ["app/pages/**/*.vue"], rules: { "vue/no-inline-style": "off" } }
];
fs.mkdirSync(configDir, { recursive: true });
fs.writeFileSync(
  path.join(configDir, "oxlint.config.json"),
  renderNuxtOxlintConfig(items, "oxlint-plugin-vize", { rootDir, configDir })
);
```

`app/pages/about.vue` and `app/components/InfoCard.vue` (same content)

```vue
<script setup lang="ts">
const title = "About";
</script>

<template>
  <h1 style="color: red">{{ title }}</h1>
</template>
```

Generated `.nuxt/oxlint.config.json`:

```json
{
  "plugins": ["vue"],
  "jsPlugins": [{ "name": "vize", "specifier": "oxlint-plugin-vize" }],
  "settings": { "vize": { "preset": "incremental" } },
  "ignorePatterns": ["../**/dist"],
  "overrides": [
    { "files": ["../**/*.vue"], "rules": { "vize/vue/no-inline-style": "warn" } },
    { "files": ["../app/pages/**/*.vue"], "rules": { "vize/vue/no-inline-style": "off" } }
  ]
}
```

```sh
node render.mjs
oxlint -c .nuxt/oxlint.config.json app/pages/about.vue app/components/InfoCard.vue
```

## Actual

Oxlint 1.75.0 and 1.86.0:

```
Failed to parse oxlint configuration file.

  x invalid config file /path/to/app/.nuxt/oxlint.config.json: Invalid pattern `../**/dist` in `ignorePatterns`: `..` is not supported, patterns are resolved within the config file's directory
```

The same config with `ignorePatterns` removed loads, but reports nothing at all: `../**/*.vue`
does not match `app/components/InfoCard.vue`, so the rule is never enabled.

For comparison, from `.nuxt/` only `**/`-prefixed globs match project files: with
`"files": ["**/*.vue"]` both files are reported, and `"files": ["**/app/pages/**/*.vue"]` turns the
rule off for `about.vue` (this is what #7251 suggested).

## Expected

The generated config loads, and its ignores and overrides apply to the files they name. Since
Oxlint resolves patterns "within the config file's directory" and rejects `..`, either:

- emit `**/`-prefixed globs (`**/app/pages/**/*.vue`, `**/dist`), as suggested in #7251, or
- write the generated config next to the project root (e.g. a gitignored root file), so
  root-relative globs work unchanged.
