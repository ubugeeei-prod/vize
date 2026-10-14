<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useSerial } from "@vizejs/composable/use-serial";

const mounted = useMounted();
const lines = ref<string[]>([]);
const { supported, connected, requestPort, open, close } = useSerial({
  decode: "text",
  onData: (chunk) => {
    lines.value = [...lines.value, String(chunk)].slice(-5);
  },
});

async function connectBoard() {
  const port = await requestPort();
  if (port) await open(port, { baudRate: 115200 });
}
</script>

<template>
  <div class="composable-example">
    <p>Read log lines from a microcontroller over a serial port.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported || connected" @click="connectBoard">
        Connect a board
      </button>
      <button type="button" :disabled="!connected" @click="() => close()">Disconnect</button>
    </div>
    <pre aria-label="Serial output">{{ lines.join("\n") }}</pre>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? connected
            ? "Reading from the board."
            : "Ready to connect."
          : "Web Serial is not available in this browser."
    }}</output>
  </div>
</template>
