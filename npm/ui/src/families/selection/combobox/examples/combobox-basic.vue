<!-- Editable combobox that filters a list of time zones as the user types. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  Combobox,
  ComboboxAnchor,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxItemIndicator,
  ComboboxTrigger,
} from "../combobox.ts";

const timeZones = [
  "Europe/London",
  "Europe/Berlin",
  "Asia/Tokyo",
  "America/New_York",
  "Australia/Sydney",
];
const timeZone = ref<string | null>(null);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Time zone</span>
    <Combobox v-slot="{ filteredItems }" v-model="timeZone" :items="timeZones" name="timeZone">
      <ComboboxAnchor>
        <ComboboxInput :aria-labelledby="labelId" placeholder="Search time zones" />
        <ComboboxTrigger />
      </ComboboxAnchor>
      <ComboboxContent>
        <ComboboxItem v-for="zone in filteredItems" :key="zone" :value="zone">
          {{ zone }}
          <ComboboxItemIndicator>✓</ComboboxItemIndicator>
        </ComboboxItem>
        <ComboboxEmpty>No matching time zones</ComboboxEmpty>
      </ComboboxContent>
    </Combobox>
    <output>Selected: {{ timeZone ?? "none" }}</output>
  </div>
</template>
