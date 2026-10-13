<script setup lang="ts">
import { useSupported } from "@vizejs/composable/use-supported";

const features = [
  { name: "Web Share", supported: useSupported(() => "share" in navigator) },
  { name: "Clipboard", supported: useSupported(() => "clipboard" in navigator) },
  { name: "View transitions", supported: useSupported(() => "startViewTransition" in document) },
];
</script>

<template>
  <div class="composable-example">
    <p>Check browser features after mounting, so server and client render the same markup.</p>
    <ul aria-label="Browser features">
      <li v-for="feature in features" :key="feature.name">
        {{ feature.name }}: {{ feature.supported.value ? "available" : "not available" }}
      </li>
    </ul>
  </div>
</template>
