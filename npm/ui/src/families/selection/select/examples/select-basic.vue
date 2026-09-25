<!-- Controlled select-only combobox with a placeholder, a labelled group, and selection marks. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectItemIndicator,
  SelectLabel,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from "../select.ts";

const timezones = ["Asia/Tokyo", "Europe/Paris", "America/New_York"];
const timezone = ref<string | null>(null);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Time zone</span>
    <Select v-model="timezone" :items="timezones" name="timezone" placeholder="Choose a time zone">
      <SelectTrigger :aria-labelledby="labelId"><SelectValue /></SelectTrigger>
      <SelectContent>
        <SelectViewport>
          <SelectGroup>
            <SelectLabel>Common zones</SelectLabel>
            <SelectItem v-for="zone in timezones" :key="zone" :value="zone">
              {{ zone }}
              <SelectItemIndicator>✓</SelectItemIndicator>
            </SelectItem>
          </SelectGroup>
        </SelectViewport>
      </SelectContent>
    </Select>
  </div>
</template>
