<!-- Draft watermark over a button that must stay clickable. -->
<script setup lang="ts">
import { ref } from "vue";

import { Watermark } from "../index.ts";

const published = ref(0);
</script>

<template>
  <Watermark text="DRAFT" :opacity="0.2">
    <button type="button" @click="published += 1">Publish {{ published }}</button>
  </Watermark>
</template>
