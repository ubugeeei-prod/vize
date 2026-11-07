# Fresco Virtual Scrolling

`ScrollBox` renders large terminal collections with work bounded by the viewport instead of the
collection size. It composes `useVirtualViewport`, `useInput`, `useMouse`, and `ScrollBar` through
the public `@vizejs/fresco` API.

## Basic usage

```vue
<script setup lang="ts">
import { ref } from "vue";
import { ScrollBox, Text } from "@vizejs/fresco";

const offset = ref(0);
const rows = Array.from({ length: 100_000 }, (_, index) => `row ${index}`);
</script>

<template>
  <ScrollBox v-model="offset" :item-count="rows.length" :viewport-size="20" :overscan="2" focused>
    <template #start><Text bold>Results</Text></template>
    <template #default="{ index }"
      ><Text>{{ rows[index] }}</Text></template
    >
    <template #end
      ><Text dim>{{ offset + 1 }} / {{ rows.length }}</Text></template
    >
  </ScrollBox>
</template>
```

The `start` and `end` slots remain mounted outside the virtual viewport. Set `follow-output` for a
log-style view that stays at the tail only while the user is already at the tail.

## Controls and public API

When `focused` is true, Up/Down, Page Up/Page Down, and Home/End update the offset. Mouse-wheel
events also update it when the app is created with `mouse: true`. `v-model` exposes the controlled
offset and the `scroll` event emits the new `VirtualViewportRange`.

Template refs expose:

- `scrollTo(offset)`
- `scrollBy(delta)`
- `scrollIntoView(index, "nearest" | "start" | "center" | "end")`
- `range()`

`useVirtualViewport` exposes the same range model for custom components. Its visible end and
overscan end are exclusive.

## Bounded rendering contract

At most `viewportSize + 2 * overscan` item slots are retained away from collection boundaries.
The behavior test exercises a 100,000-item collection with a five-row viewport and two rows of
overscan, and verifies that no more than nine item nodes are retained after scrolling.

This is Vue-layer virtualization, not yet a general native clipping primitive. Content created by
the default slot is culled before it reaches the native render tree, while arbitrary translated or
overlapping native content still awaits the M1 clipping and routed-input work. Mouse input is also
focus-gated and global until native hit-testing and capture/bubble routing land.

See [`npm/fresco/examples/virtual-list.vue`](../../npm/fresco/examples/virtual-list.vue) for the
canonical SFC example and `virtual-list-main.ts` for its executable entry.
