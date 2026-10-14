<script setup lang="ts">
import { useGamepad } from "@vizejs/composable/use-gamepad";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, gamepads } = useGamepad();
</script>

<template>
  <div class="composable-example">
    <p>Show connected game controllers and which buttons are pressed.</p>
    <ul aria-label="Controllers">
      <li v-for="pad in gamepads" :key="pad.index">
        {{ pad.id }}: {{ pad.buttons.filter((button) => button.pressed).length }} buttons pressed
      </li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "The Gamepad API is not available in this browser."
          : gamepads.length === 0
            ? "Connect a controller and press any button."
            : `${gamepads.length} controllers connected.`
    }}</output>
  </div>
</template>
