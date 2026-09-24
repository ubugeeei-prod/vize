export const qrCodeRendererFixtures = [
  {
    filename: "QrCodeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { QrCode } from "./families/media/qr-code/qr-code.ts";

const url = ref("https://vizejs.dev");
</script>

<template>
  <QrCode :value="url" error-correction="H" label="Vize website" :quiet-zone="4">
    <template #overlay="{ dimension }">
      <rect :x="dimension / 2 - 3" :y="dimension / 2 - 3" width="6" height="6" fill="white" />
    </template>
    <template #fallback="{ error }">
      <span>{{ error?.code }}</span>
    </template>
  </QrCode>
</template>
`,
  },
] as const;
