<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useScreenDetails } from "@vizejs/composable/use-screen-details";

const mounted = useMounted();
const { supported, isExtended, screens, request } = useScreenDetails();
</script>

<template>
  <div class="composable-example">
    <p>Open speaker notes on a second screen when one is connected.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => request()">
      Find screens
    </button>
    <ul aria-label="Screens">
      <li v-for="(screen, index) in screens" :key="screen.label || index">
        {{ screen.label || `Screen ${index + 1}` }}
      </li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "Window Management is not available; open the notes in a new tab."
          : isExtended
            ? "A second screen is connected."
            : "One screen detected."
    }}</output>
  </div>
</template>
