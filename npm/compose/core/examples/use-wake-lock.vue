<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useWakeLock } from "@vizejs/composable/use-wake-lock";

const mounted = useMounted();
const { supported, active, error, request, release } = useWakeLock();
</script>

<template>
  <div class="composable-example">
    <p>Keep the screen on while a recipe or a slide deck is open.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported || active" @click="() => request()">
        Keep screen on
      </button>
      <button type="button" :disabled="!active" @click="() => release()">Allow sleep</button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "Screen Wake Lock is not available in this browser."
          : active
            ? "The screen stays on."
            : error
              ? "The browser did not keep the screen on."
              : "The screen may turn off."
    }}</output>
  </div>
</template>
