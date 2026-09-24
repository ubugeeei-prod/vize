<!-- Multi-select listbox whose options toggle and render a decorative selection indicator. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import { Listbox, ListboxItem, listboxSelectedValues, type ListboxValue } from "../listbox.ts";

const toppings = ref<ListboxValue>(["basil"]);
const selected = computed(() => listboxSelectedValues(toppings.value));
const options = [
  { value: "basil", label: "Basil" },
  { value: "mushroom", label: "Mushroom" },
  { value: "olive", label: "Olive" },
];
</script>

<template>
  <div>
    <Listbox v-model="toppings" selection-mode="multiple" aria-label="Pizza toppings">
      <ListboxItem
        v-for="option in options"
        :key="option.value"
        :value="option.value"
        :text-value="option.label"
      >
        <template #default>{{ option.label }}</template>
        <template #indicator="{ selected: isSelected }">
          <span aria-hidden="true">{{ isSelected ? "✓" : "" }}</span>
        </template>
      </ListboxItem>
    </Listbox>
    <output>{{ selected.length }} selected: {{ selected.join(", ") || "none" }}</output>
  </div>
</template>
