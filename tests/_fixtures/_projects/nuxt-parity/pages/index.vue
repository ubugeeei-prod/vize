<script setup lang="ts">
// Auto-imported composable (no import statement) -> must resolve to the real
// typed `useCounter`, not an `any` stub.
const counter = useCounter(2);
counter.increment(1);

// Auto-imported framework global.
const route = useRoute();

// Path alias `~/utils/format` declared in `.nuxt/tsconfig.json`.
import { shout } from "~/utils/format";

const heading = shout(route.path);
</script>

<template>
  <!-- Auto-registered component <TheGreeting> from `.nuxt/components.d.ts`. -->
  <main>
    <TheGreeting :name="heading" />
    <p>Count: {{ counter.count }}</p>
  </main>
</template>
