<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { usePermission } from "@vizejs/composable/use-permission";

const mounted = useMounted();
const { state, supported } = usePermission("geolocation");
</script>

<template>
  <div class="composable-example">
    <p>
      Explain a location feature differently depending on whether the reader already allowed it.
    </p>
    <output aria-live="polite">{{
      !mounted
        ? "Checking permissions…"
        : !supported
          ? "This browser cannot report permission states."
          : state === "granted"
            ? "Location is allowed; nearby venues load automatically."
            : state === "denied"
              ? "Location is blocked; search for a venue instead."
              : "Location: we will ask when you look for nearby venues."
    }}</output>
  </div>
</template>
