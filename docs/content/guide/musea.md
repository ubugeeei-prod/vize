---
title: Musea
---

# Musea

> **⚠️ Work in Progress:** Musea is still evolving. File formats, APIs, and UI behavior may change.

Musea is Vize's art-file and component-gallery toolchain.

- `vize_musea` is the Rust core for parsing `*.art.vue`, generating docs, building prop palettes,
  autogenerating variants, and preparing VRT data.
- `@vizejs/vite-plugin-musea` is the recommended gallery and dev-server workflow today.
- `musea-vrt` is the CLI for visual regression snapshots, a11y audits, approvals, cleanup, and
  generated art files.

## Overview

![Musea Component Gallery — Home](/musea-home.png)

Musea uses `*.art.vue` files to describe component variants with Vue-native syntax.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the package:

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Recommended Usage: Vite Plugin

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

Run your normal Vite dev server and open the configured Musea route:

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

If you install the `vize` npm package, `vp exec vize musea` is a convenience wrapper around Vite:

```bash
vp exec vize musea
vp exec vize musea --build
```

## Shared Config

`musea()` options override shared config. Put stable project defaults in `vize.config.ts` and keep
preview-only settings in `vite.config.ts`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

Shared config currently covers `include`, `exclude`, `basePath`, `storybookCompat`, and
`inlineArt`. Pass `previewCss`, `previewSetup`, `tokensPath`, `theme`, and `storybookOutDir`
directly to `musea()`.

## Art Files

```art-vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./MyButton.vue", {
  title: "MyButton",
  category: "Components",
  status: "ready",
  tags: ["button", "ui", "input"],
});

const pressed = ref(false);
</script>

<art>
  <variant name="Default" default>
    <MyButton type="button" :pressed="pressed">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined :pressed="pressed">Click me</MyButton>
  </variant>
</art>
```

`defineArt(source, options)` is a compiler macro. It declares the component that Musea should load,
plus metadata that used to live on `<art>`. Prefer a relative component path string such as
`defineArt("./MyButton.vue", { title: "MyButton" })`; Musea imports that component in generated
runtime code and the language server uses the same source for prop and slot inference.
The source string participates in path completion, unresolved-file diagnostics, document links, and
go-to-definition.

`<art title="..." component="...">` still works for compatibility, and explicit `<art>` attributes
override `defineArt` metadata when both are present.

### Variant-local state

Root `<script setup>` state is isolated per variant by default. Each variant receives its own setup
instance, so refs and computed values in one variant do not leak into another:

```art-vue
<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
```

Use `<script setup isolate="false">` only when the art file intentionally needs one shared setup
instance across every variant:

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

### Anatomy

| Element / Macro                  | Purpose                                |
| -------------------------------- | -------------------------------------- |
| `defineArt(source, options)`     | Target component and art metadata      |
| `defineArt(...).title`           | Display name                           |
| `defineArt(...).category`        | Sidebar grouping                       |
| `defineArt(...).status`          | Optional status badge                  |
| `defineArt(...).tags`            | Search and filtering tags              |
| `<script setup>`                 | Variant-local setup state by default   |
| `<script setup isolate="false">` | Shared setup state across all variants |
| `<art>`                          | Root variants block                    |
| `<art title component ...>`      | Compatibility metadata attributes      |
| `<variant>`                      | Named component variation              |
| `default`                        | Marks the default variant              |
| `args`, `viewport`, `skip-vrt`   | Optional variant configuration         |

Keep art files close to the component when variants are part of the component's contract:

```txt
src/components/Button.vue
src/components/Button.art.vue
```

Use a separate `stories` or `art` directory when a design system owns many cross-cutting examples:

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## Inline Art

When `inlineArt` is enabled, regular `.vue` files that contain an `<art>` block can appear in the
gallery. This is useful for small components where examples should live in the same file.

```ts
musea({
  inlineArt: true,
});
```

Inside inline art, use `<Self>` to render the host component.

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

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

## Preview Configuration

You can inject project CSS and preview setup code:

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

This is useful for installing plugins such as `vue-i18n` or `vue-router` in the preview iframe.

```ts
// musea.preview.ts
import type { App } from "vue";
import { createI18n } from "vue-i18n";

export default function setup(app: App) {
  app.use(
    createI18n({
      legacy: false,
      locale: "en",
      messages: {
        en: {},
      },
    }),
  );
}
```

## Visual Regression Testing

The package exposes the `musea-vrt` binary:

```bash
vp exec musea-vrt --base-url http://localhost:5173
vp exec musea-vrt --update
vp exec musea-vrt --ci --json
vp exec musea-vrt --a11y
vp exec musea-vrt approve
vp exec musea-vrt approve "Button/*"
vp exec musea-vrt clean
```

Typical CI flow starts the Vite server in one process, then runs the snapshot command against it:

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

Use `--update` locally to refresh baselines, `approve` to accept failed snapshots, and `clean` to
remove orphaned snapshots after deleting variants.

## Generate Art Files

Use the generator to create a first `.art.vue` draft from an existing component:

```bash
vp exec musea-vrt generate src/components/Button.vue
```

The generated file is a starting point. Review the variants, titles, tags, and prop coverage before
committing it.

## Storybook Output

Enable Storybook-compatible CSF generation when you want Musea art files to feed a Storybook setup:

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## CLI Status

`vize musea` exists in the Rust CLI, but the recommended Musea workflow today is still the Vite
plugin path. Treat the Rust subcommand as experimental while the dedicated gallery workflow settles.

The Rust subcommand can scaffold a starter art project:

```bash
vize musea new
```

## Related Packages

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
