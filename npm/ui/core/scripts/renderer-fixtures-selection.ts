export const selectionRendererFixtures = [
  {
    filename: "ListboxConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Listbox, ListboxItem } from "./listbox.ts";
</script>

<template>
  <Listbox aria-label="Favorite fruit" default-value="apple">
    <ListboxItem value="apple" text-value="Apple">Apple</ListboxItem>
    <ListboxItem value="pear" text-value="Pear">Pear</ListboxItem>
  </Listbox>
</template>
`,
  },
] as const;
