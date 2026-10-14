<script setup lang="ts">
import { useUserActivation } from "@vizejs/composable/use-user-activation";

const { hasBeenActive } = useUserActivation();
</script>

<template>
  <div class="composable-example">
    <p>Wait for the reader's first interaction before playing sound or opening a popup.</p>
    <button type="button">Interact with the page</button>
    <output aria-live="polite">{{
      hasBeenActive ? "The page may now play sound." : "Waiting for the first interaction."
    }}</output>
  </div>
</template>
