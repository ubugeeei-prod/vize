<!-- Popover dialog with a labelled heading and a form field, opened via v-model:open. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { PopoverArrow, PopoverContent, PopoverRoot, PopoverTrigger } from "../popover.ts";

const open = ref(false);
const width = ref("1280");
const headingId = useId();
const inputId = useId();
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger>Canvas size</PopoverTrigger>
    <PopoverContent placement="bottom-start" :aria-labelledby="headingId">
      <h3 :id="headingId">Canvas size</h3>
      <label :for="inputId">Width in pixels</label>
      <input :id="inputId" v-model="width" type="number" min="1" />
      <button type="button" @click="() => (open = false)">Done</button>
      <PopoverArrow />
    </PopoverContent>
  </PopoverRoot>
</template>
