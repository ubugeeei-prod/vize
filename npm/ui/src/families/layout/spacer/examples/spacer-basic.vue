<!-- Decorative spacers that add block and inline gaps without extra markup semantics. -->
<script setup lang="ts">
import { Spacer } from "../spacer.ts";
</script>

<template>
  <div>
    <h2>Checkout</h2>
    <Spacer size="1.5rem" />
    <p>
      <button type="button">Back</button>
      <Spacer axis="inline" size="0.75rem" />
      <button type="button">Place order</button>
    </p>
  </div>
</template>
