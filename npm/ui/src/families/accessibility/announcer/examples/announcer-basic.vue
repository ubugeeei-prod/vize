<!-- Provider that owns the live regions and announces cart updates from its slot. -->
<script setup lang="ts">
import { ref } from "vue";

import { AnnouncerProvider, type AnnouncerController } from "../announcer.ts";

const count = ref(0);

function addItem(announcer: AnnouncerController): void {
  count.value += 1;
  announcer.announce(`Added to cart. ${count.value} items in cart.`, { key: "cart" });
}

function emptyCart(announcer: AnnouncerController): void {
  count.value = 0;
  announcer.announce("Cart emptied.", { politeness: "assertive" });
}
</script>

<template>
  <AnnouncerProvider v-slot="{ announcer }">
    <p>Items in cart: {{ count }}</p>
    <button type="button" @click="() => addItem(announcer)">Add to cart</button>
    <button type="button" @click="() => emptyCart(announcer)">Empty cart</button>
  </AnnouncerProvider>
</template>
