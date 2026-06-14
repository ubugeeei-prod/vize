# @vizejs/vite-plugin-musea

Vite plugin for Musea - Vue component gallery and documentation.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the package:

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

Run your Vite dev server and open the gallery route:

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

Musea middleware is intended for the local Vite dev server and trusted development networks. Do
not expose `/__musea__` directly on the public internet unless it is protected by your own network
controls or authentication layer.

Shared defaults can live in `vize.config.ts`:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    inlineArt: false,
    storybookCompat: false,
  },
});
```

Direct `musea()` options override shared config. Pass preview-only options such as `previewCss`,
`previewSetup`, `tokensPath`, `theme`, and `storybookOutDir` directly to the plugin.

## Art File Format

```vue
<!-- Button.art.vue -->
<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  category: "UI",
  tags: ["button", "form"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Disabled</Button>
  </variant>
</art>
```

`defineArt(source, options)` is a compiler macro. It declares the Vue component Musea should render
and the metadata shown in the gallery. Prefer a relative component path string; the macro call is
removed and Musea generates the component import. The Musea language server uses the same source
for path completion, missing-file diagnostics, go-to-definition, and prop/slot inference.

## TypeScript and Editor Setup

Add the client types once, usually in `src/env.d.ts`:

```ts
/// <reference types="@vizejs/vite-plugin-musea/client" />
```

If your project type-checks `.art.vue` files with Volar or `vue-tsc`, include the extension in
`tsconfig.json`:

```json
{
  "include": ["src/**/*.ts", "src/**/*.vue", "src/**/*.art.vue"],
  "vueCompilerOptions": {
    "extensions": [".vue", ".art.vue"]
  }
}
```

Root `<script setup>` state is variant-local by default, so each variant gets its own refs,
computed values, and composable calls:

```vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
</script>

<art>
  <variant name="Initial" default>
    <Counter :count="count" />
  </variant>
  <variant name="Interactive">
    <Counter :count="count" />
  </variant>
</art>
```

Use `<script setup isolate="false">` when variants intentionally share one setup instance.

Legacy `<art>` metadata attributes are still supported:

| Attribute           | Purpose                               |
| ------------------- | ------------------------------------- |
| `title`             | Display name in the gallery           |
| `component`         | Relative source component path        |
| `category`          | Sidebar grouping                      |
| `status`            | Optional status badge                 |
| `tags`              | Search and filtering tags             |
| `action-events`     | Comma-separated events to capture     |
| `capture-mousemove` | Include mousemove in captured actions |

Enable inline art when examples should live inside the component file:

```ts
musea({
  inlineArt: true,
});
```

Use `<Self>` in an inline `<art>` block to render the host component.

## Preview Setup

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

```ts
// musea.preview.ts
import type { App } from "vue";

export default function setup(app: App) {
  // Install vue-router, vue-i18n, stores, or design-system plugins here.
}
```

## Design Tokens

Expose a Style Dictionary-compatible token file in the gallery:

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

Tailwind v4 theme variables can also be used as the token source:

```css
/* src/styles/main.css */
@import "tailwindcss";

@theme {
  --color-brand: oklch(70.5% 0.213 47.604);
  --color-accent: var(--color-brand);
  --spacing-card: 1.5rem;
}
```

```ts
musea({
  tokensPath: "src/styles/main.css",
});
```

`tokensPath` reads tokens for the gallery and token APIs. Use `previewCss` separately when that
CSS file should also be loaded inside component preview iframes.

## Commands

```bash
# Start dev server
vp dev

# Build gallery
vp build

# Run visual regression snapshots
vp exec musea-vrt --base-url http://localhost:5173

# Update local baselines
vp exec musea-vrt --update

# CI mode with JSON output
vp exec musea-vrt --ci --json

# Run a11y audits alongside snapshots
vp exec musea-vrt --a11y

# Approve failed snapshots
vp exec musea-vrt approve

# Remove orphaned snapshots
vp exec musea-vrt clean

# Generate an art-file draft
vp exec musea-vrt generate src/components/Button.vue
```

## License

MIT
