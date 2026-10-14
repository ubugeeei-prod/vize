<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useUSB } from "@vizejs/composable/use-usb";

const mounted = useMounted();
const { supported, devices, requestDevice } = useUSB();
</script>

<template>
  <div class="composable-example">
    <p>Let the reader grant access to a USB device, such as a label printer.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => requestDevice([])">
      Choose a USB device
    </button>
    <ul aria-label="Allowed devices">
      <li v-for="item in devices" :key="`${item.vendorId}-${item.productId}`">
        {{ item.productName }}
      </li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `${devices.length} devices allowed.`
          : "WebUSB is not available in this browser."
    }}</output>
  </div>
</template>
