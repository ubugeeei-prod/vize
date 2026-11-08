# Migrate an Ink application to Fresco

Fresco keeps Ink's component vocabulary where the terminal semantics match, but replaces React
with Vue SFCs and Composition API state. Migrate one screen at a time: preserve the application
model first, then move rendering and input ownership.

## Runtime entrypoint

Replace Ink's `render()` lifecycle with the public Fresco application contract:

```ts
import { createApp } from "@vizejs/fresco";
import App from "./App.vue";

const app = createApp(App, { exitOnCtrlC: true });
await app.mount();
await app.waitUntilExit();
```

The returned application owns terminal acquisition and restoration. Do not import the custom
renderer or native binding directly.

## Component mapping

| Ink                          | Fresco                         | Migration note                                                     |
| ---------------------------- | ------------------------------ | ------------------------------------------------------------------ |
| `Box`                        | `Box`                          | Flex, border, spacing, positioning, and overflow props stay typed. |
| `Text`                       | `Text`                         | Use `wrap` for wrapping and truncation policy.                     |
| `Newline`                    | `Newline`                      | Canonical implementation is an authored SFC.                       |
| `Spacer`                     | `Spacer`                       | Expands on the active flex axis.                                   |
| `Static`                     | `Static`                       | Retains completed output above the live tree.                      |
| `Transform`                  | `Transform`                    | Receives each rendered line and its index.                         |
| `useInput`                   | `useInput` / routed host input | Global subscriptions exist; stable host nodes also route events.   |
| `useApp`                     | `useApp`                       | Exposes exit, render, clear, width, and height.                    |
| ecosystem text/select inputs | Fresco typed components        | Consult the compatibility matrix for remaining ownership gaps.     |

## Convert a component

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { Box, Text, useInput } from "@vizejs/fresco";

const count = ref(0);
const label = computed(() => `Count: ${count.value}`);

useInput({
  onChar(character) {
    if (character === "+") count.value += 1;
  },
});
</script>

<template>
  <Box border="rounded" :padding="1">
    <Text :bold="true" fg="cyan">{{ label }}</Text>
  </Box>
</template>
```

Keep stable application keys when converting mapped lists. Vue reuses the corresponding Fresco
host nodes across keyed moves, so lifecycle observers and routed events retain their node IDs.

## Known differences

- Fresco render nodes form a closed JS/native protocol; use public components or `FrameBuffer`
  instead of importing renderer internals.
- Input routing supports capture, target, and bubble phases, while several higher-level controls
  still require their host application to own navigation. The compatibility matrix identifies
  each partial surface.
- Terminal dimensions and styling use cells and terminal colors rather than browser CSS pixels.
- The native package and JavaScript package must use matching versions.

Use the [compatibility matrix](./compatibility.md) as the release-specific source of truth.
