# Vize Art - VS Code Extension

Syntax highlighting for Vize Art files (`*.art.vue`).

Install the `Vize` extension alongside this one if you also want hover, completion,
go-to-definition, and references inside `*.art.vue`.

## Features

- Syntax highlighting for `<art>` and `<variant>` blocks
- Syntax highlighting for root `<script setup>` and the `isolate` attribute
- Highlighting for Art-specific attributes (`title`, `component`, `description`, etc.)
- Highlighting for variant attributes (`name`, `default`, `props`, `attrs`, `play`, etc.)
- Embedded language support for `<script>` (TypeScript) and `<style>` (CSS)
- Vue template syntax highlighting inside variants

## Art File Format

```vue
<!-- Button.art.vue -->
<script setup lang="ts">
import Button from "./Button.vue";

defineArtMeta({
  category: "UI",
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Click me</Button>
  </variant>
</art>
```

Musea infers the target from the imported component tag shared by every variant. `defineArtMeta()`
is optional, as is `title`; the title defaults to the inferred component binding or filename.
Use `defineArt(Button, metadata?)` only when an intentionally ambiguous composition needs an
explicit target. Root `<script setup>` state is isolated per variant by default; add
`isolate="false"` to share one setup instance across all variants.

## Installation

Install the extension from the VS Code Marketplace as `vize.vize-art`.

### From Marketplace

```bash
code --install-extension vize.vize-art
```

### Development

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then:

```bash
cd editors/vscode-art
vp install --ignore-workspace
vp exec tsgo -p ./
```

Then press F5 in VS Code to launch the Extension Development Host.

## Supported Attributes

### Art Block (`<art>`)

- `title` - Optional component-title override
- `component` - Compatibility path override
- `description` - Component description
- `category` - Category for grouping
- `tags` - Comma-separated tags
- `status` - Development status (stable, beta, deprecated)
- `order` - Display order

### Variant Block (`<variant>`)

- `name` - Variant name (required)
- `default` - Boolean flag for default variant
- `props` - Compatibility object for component prop values; direct component attributes are preferred
- `attrs` - Compatibility object for fallthrough attributes; direct component attributes are preferred
- `play` - Typed interaction test
- `viewport` - Viewport preset
- `skip-vrt` - Skip visual regression testing

## License

MIT
