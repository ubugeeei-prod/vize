## Stamp the page without blocking it

`Watermark` repeats text in an SVG data URI. The overlay is `aria-hidden` and
`pointer-events: none`, so the slot stays clickable and is not read twice.
The image is built during setup, including on the server, and does not measure
the document.

```vue
<script setup lang="ts">
import { Watermark } from "@vizejs/ui/watermark";
</script>

<template>
  <Watermark text="DRAFT">
    <button type="button">Publish</button>
  </Watermark>
</template>
```

`text` is escaped. `<` in the label cannot break out of the SVG. `gap` is the
tile size and `rotate` is the clockwise angle of each tile.
