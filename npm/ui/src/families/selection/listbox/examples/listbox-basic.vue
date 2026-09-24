<!-- Single-select listbox bound with v-model, labelled by a heading, with one disabled option. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Listbox, ListboxItem, type ListboxValue } from "../listbox.ts";

const region = ref<ListboxValue>("eu-west");
const labelId = useId();
const regions = [
  { value: "us-east", label: "US East (Virginia)", disabled: false },
  { value: "eu-west", label: "EU West (Ireland)", disabled: false },
  { value: "ap-south", label: "Asia Pacific (Mumbai)", disabled: true },
];
</script>

<template>
  <div>
    <h3 :id="labelId">Deployment region</h3>
    <Listbox v-model="region" :aria-labelledby="labelId">
      <ListboxItem
        v-for="item in regions"
        :key="item.value"
        :value="item.value"
        :disabled="item.disabled"
      >
        {{ item.label }}
      </ListboxItem>
    </Listbox>
    <output>Selected: {{ region ?? "none" }}</output>
  </div>
</template>
