<script setup lang="ts">
import { useHID } from "@vizejs/composable/use-hid";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, devices, requestDevice } = useHID();
</script>

<template>
  <div class="composable-example">
    <p>Connect a USB or Bluetooth HID device, such as a macro keypad.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => requestDevice([])">
      Choose a device
    </button>
    <ul aria-label="Connected devices">
      <li v-for="item in devices" :key="`${item.vendorId}-${item.productId}`">
        {{ item.productName }}
      </li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `${devices.length} devices allowed.`
          : "WebHID is not available in this browser."
    }}</output>
  </div>
</template>
