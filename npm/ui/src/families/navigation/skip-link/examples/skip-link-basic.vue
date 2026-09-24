<!-- Skip link that moves focus past navigation to the main content region. -->
<script setup lang="ts">
import { useId } from "vue";

import { SkipLink } from "../skip-link.ts";

const mainId = useId();
</script>

<template>
  <div>
    <SkipLink :href="`#${mainId}`">Skip to main content</SkipLink>
    <nav aria-label="Primary">
      <a href="#home">Home</a>
      <a href="#pricing">Pricing</a>
    </nav>
    <main :id="mainId">
      <h1>Welcome back</h1>
    </main>
  </div>
</template>
