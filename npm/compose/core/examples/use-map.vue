<script setup lang="ts">
import { computed } from "vue";
import { useMap } from "@vizejs/composable/use-map";

const products = ["Sticker", "T-shirt", "Mug"];
const cart = useMap<string, number>();
const total = computed(() => cart.entries.value.reduce((sum, [, count]) => sum + count, 0));

function add(product: string) {
  cart.update(product, (count) => (count ?? 0) + 1);
}
</script>

<template>
  <div class="composable-example">
    <p>Count items in a cart, one entry per product.</p>
    <div class="example-actions">
      <button v-for="product in products" :key="product" type="button" @click="() => add(product)">
        Add {{ product }}
      </button>
      <button type="button" @click="() => cart.clear()">Empty cart</button>
    </div>
    <ul aria-label="Cart">
      <li v-for="[product, count] in cart.entries.value" :key="product">
        {{ product }} × {{ count }}
      </li>
    </ul>
    <output aria-live="polite">{{ cart.size.value }} products, {{ total }} items</output>
  </div>
</template>
