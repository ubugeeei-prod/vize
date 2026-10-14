<script setup lang="ts">
import { useBluetooth } from "@vizejs/composable/use-bluetooth";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, device, connected, error, requestDevice, connect, disconnect } = useBluetooth({
  requestOptions: { filters: [{ services: ["heart_rate"] }] },
});

async function pair() {
  if (await requestDevice()) await connect();
}
</script>

<template>
  <div class="composable-example">
    <p>Pair a Bluetooth heart-rate monitor, or explain why the browser cannot.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported || connected" @click="pair">
        Pair a monitor
      </button>
      <button type="button" :disabled="!connected" @click="() => disconnect()">Disconnect</button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "Web Bluetooth is not available in this browser."
          : connected
            ? `Connected to ${device?.name ?? "the monitor"}.`
            : error
              ? "Pairing was cancelled or failed."
              : "Ready to pair."
    }}</output>
  </div>
</template>
