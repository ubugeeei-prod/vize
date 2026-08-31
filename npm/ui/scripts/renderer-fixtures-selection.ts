export const selectionRendererFixtures = [
  {
    filename: "NativeSelectConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { NativeSelect } from "./families/selection/native-select/native-select.ts";

const options = [
  { label: "Apple", value: "apple" },
  { label: "Pear", value: "pear" },
] as const;
</script>

<template>
  <NativeSelect aria-label="Favorite fruit" default-value="apple" :options="options" />
</template>
`,
  },
  {
    filename: "ListboxConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Listbox, ListboxItem } from "./families/selection/listbox/listbox.ts";
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
