<script setup lang="ts">
import { ref } from "vue";
import { useBarcodeDetector } from "@vizejs/composable/use-barcode-detector";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, detect } = useBarcodeDetector({ formats: ["qr_code"] });
const found = ref<string>();

async function scan(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const codes = await detect(await createImageBitmap(file));
  found.value = codes[0]?.rawValue ?? "No QR code found in the image.";
}
</script>

<template>
  <div class="composable-example">
    <p>Read a QR code from a photo of a conference badge.</p>
    <label>
      Badge photo
      <input type="file" accept="image/*" :disabled="!mounted || !supported" @change="scan" />
    </label>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "The Barcode Detection API is not available in this browser."
          : (found ?? "Choose a photo to scan.")
    }}</output>
  </div>
</template>
