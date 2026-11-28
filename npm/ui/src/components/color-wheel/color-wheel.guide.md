## A clockwise hue, with 0 at the right

`ColorWheel` is a slider for hue. The geometry matches a screen: 0° is the
right-hand point, and the angle increases clockwise because y grows downward.
Arrow keys step by 1°. Shift and Page Up / Page Down step by 10°. Home is 0.
End is 360, the same color as 0, so a full turn is representable.

```vue
<script setup lang="ts">
import { ref } from "vue";

import { ColorWheel } from "@vizejs/ui/color-wheel";

const hue = ref(200);
</script>

<template>
  <ColorWheel v-model="hue" label="Hue" />
</template>
```

The root sets `--vize-color-wheel-hue`. The marker is `[data-part="marker"]`.
Dragging uses pointer capture. Nothing reads `window` during setup, so the
marker position is stable across SSR and hydration.
