<!-- Controlled bold-formatting toggle bound with v-model. -->
<script setup lang="ts">
import { ref } from "vue";

import { Toggle } from "../toggle.ts";

const bold = ref(false);
</script>

<template>
  <div>
    <Toggle v-model="bold" aria-label="Bold">
      <template #default="{ pressed }">
        <strong>B</strong>
        <span>{{ pressed ? "on" : "off" }}</span>
      </template>
    </Toggle>
    <output>{{ bold ? "Bold text enabled" : "Bold text disabled" }}</output>
  </div>
</template>
