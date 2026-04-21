---
title: Musea
---

# Musea

> **⚠️ Work in Progress:** Musea is still evolving. File formats, APIs, and UI behavior may change.

Musea is Vize's art-file and component-gallery toolchain.

- `vize_musea` is the Rust core for parsing `*.art.vue`, generating docs, building prop palettes,
  autogenerating variants, and preparing VRT data.
- `@vizejs/vite-plugin-musea` is the recommended gallery and dev-server workflow today.

## Overview

![Musea Component Gallery — Home](/musea-home.png)

Musea uses `*.art.vue` files to describe component variants with Vue-native syntax.

## Installation

```bash
pnpm add -D @vizejs/vite-plugin-musea
```

## Recommended Usage: Vite Plugin

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

Run your normal Vite dev server and open the configured Musea route.

## Art Files

```art-vue
<script setup lang="ts">
import MyButton from "./MyButton.vue";
</script>

<art
  title="MyButton"
  component="./MyButton.vue"
  category="Components"
  status="ready"
  tags="button, ui, input"
>
  <variant name="Default" default>
    <MyButton type="button">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined>Click me</MyButton>
  </variant>
</art>
```

### Anatomy

| Element / Attribute | Purpose                               |
| ------------------- | ------------------------------------- |
| `<art>`             | Root metadata block                   |
| `title`             | Display name                          |
| `component`         | Relative path to the source component |
| `category`          | Sidebar grouping                      |
| `status`            | Optional status badge                 |
| `tags`              | Search and filtering tags             |
| `<variant>`         | Named component variation             |
| `default`           | Marks the default variant             |

## Gallery Features

![Musea Component Detail — Variants](/musea-component.png)

Musea can surface:

- component and variant metadata
- prop palette generation
- design token views
- accessibility checks
- visual regression testing helpers
- Storybook-compatible output when requested

## Props Palette

![Musea Props Panel](/musea-props.png)

The palette pipeline can infer interactive controls from component metadata and art definitions.

## Design Tokens

![Musea Design Tokens](/musea-tokens.png)

`@vizejs/vite-plugin-musea` can ingest a Style Dictionary-compatible token file and expose it in
the gallery UI.

## Preview Configuration

You can inject project CSS and preview setup code:

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

This is useful for installing plugins such as `vue-i18n` or `vue-router` in the preview iframe.

## CLI Status

`vize musea` exists in the Rust CLI, but the recommended Musea workflow today is still the Vite
plugin path. Treat the Rust subcommand as experimental while the dedicated gallery workflow settles.

## Related Packages

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
