<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useWebMIDI } from "@vizejs/composable/use-web-midi";

const mounted = useMounted();
const { supported, status, inputs, request } = useWebMIDI();
</script>

<template>
  <div class="composable-example">
    <p>List MIDI keyboards and controllers after the reader allows access.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => request()">
      Allow MIDI access
    </button>
    <ul aria-label="MIDI inputs">
      <li v-for="input in inputs" :key="input.id">{{ input.name }}</li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `MIDI access: ${status}`
          : "Web MIDI is not available in this browser."
    }}</output>
  </div>
</template>
