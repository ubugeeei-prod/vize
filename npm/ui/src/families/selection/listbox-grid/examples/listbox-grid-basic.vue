<!-- Single-select color swatch grid navigated with arrow keys in two dimensions. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { ListboxGrid, ListboxGridItem } from "../listbox-grid.ts";

const swatches = ["Crimson", "Coral", "Gold", "Olive", "Teal", "Navy", "Plum", "Slate"];
const color = ref<string | null>("Teal");
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Label color</span>
    <ListboxGrid v-model="color" :columns="4" name="labelColor" :aria-labelledby="labelId">
      <ListboxGridItem v-for="swatch in swatches" :key="swatch" :value="swatch">
        {{ swatch }}
      </ListboxGridItem>
    </ListboxGrid>
    <output>Selected: {{ color ?? "none" }}</output>
  </div>
</template>
