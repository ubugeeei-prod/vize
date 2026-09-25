<!-- Polite live region that announces when an item is added to the cart. -->
<script setup lang="ts">
import { ref, useTemplateRef } from "vue";

import { LiveRegion } from "../live-region.ts";

const region = useTemplateRef<InstanceType<typeof LiveRegion>>("region");
const count = ref(0);

function addToCart(): void {
  count.value += 1;
  region.value?.announce(`Added to cart. ${count.value} items in your cart.`);
}
</script>

<template>
  <div>
    <button type="button" @click="addToCart">Add notebook to cart</button>
    <LiveRegion ref="region" />
  </div>
</template>
