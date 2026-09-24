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
  {
    filename: "SelectConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from "./families/selection/select/select.ts";

interface Fruit {
  readonly id: number;
  readonly name: string;
}

const fruits: readonly Fruit[] = [
  { id: 1, name: "Apple" },
  { id: 2, name: "Pear" },
];
const fruit = ref<Fruit | null>(null);
</script>

<template>
  <SelectRoot v-model="fruit" :items="fruits" by="id" name="fruit" placeholder="Pick a fruit">
    <SelectTrigger aria-label="Fruit"><SelectValue /></SelectTrigger>
    <SelectContent>
      <SelectViewport>
        <SelectItem v-for="item in fruits" :key="item.id" :value="item">
          {{ item.name }}
          <SelectItemIndicator>✓</SelectItemIndicator>
        </SelectItem>
      </SelectViewport>
    </SelectContent>
  </SelectRoot>
</template>
`,
  },
  {
    filename: "ComboboxConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import {
  ComboboxAnchor,
  ComboboxChip,
  ComboboxChipRemove,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxRoot,
} from "./families/selection/combobox/combobox.ts";

const languages: readonly string[] = ["TypeScript", "Rust", "Go"];
const picked = ref<readonly string[]>([]);
</script>

<template>
  <ComboboxRoot v-slot="{ filteredItems, selected }" v-model="picked" :items="languages" multiple>
    <ComboboxAnchor>
      <ComboboxChip v-for="language in selected" :key="language" :value="language">
        {{ language }}
        <ComboboxChipRemove>x</ComboboxChipRemove>
      </ComboboxChip>
      <ComboboxInput aria-label="Languages" />
    </ComboboxAnchor>
    <ComboboxContent>
      <ComboboxItem v-for="language in filteredItems" :key="language" :value="language">
        {{ language }}
      </ComboboxItem>
      <ComboboxEmpty>No languages</ComboboxEmpty>
    </ComboboxContent>
  </ComboboxRoot>
</template>
`,
  },
] as const;
