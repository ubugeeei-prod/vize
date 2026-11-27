<!-- Controlled hue wheel with the selected color named beside it. -->
<script setup lang="ts">
import { ref } from "vue";

import { ColorWheel } from "../index.ts";

const hue = ref(200);
</script>

<template>
  <ColorWheel v-slot="{ color, hue: current }" v-model="hue" label="Hue">
    <output>{{ current }} {{ color }}</output>
  </ColorWheel>
</template>
