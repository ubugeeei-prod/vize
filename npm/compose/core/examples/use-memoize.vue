<script setup lang="ts">
import { ref } from "vue";
import { useMemoize } from "@vizejs/composable/use-memoize";

const calculations = ref(0);
const quote = useMemoize(
  (zone: string) => {
    calculations.value += 1;
    return zone === "Europe" ? 12 : zone === "Asia" ? 18 : 25;
  },
  { getKey: (zone: string) => zone },
);
const zone = ref("Europe");
const price = ref<number>();

function getQuote() {
  price.value = quote(zone.value);
}
</script>

<template>
  <div class="composable-example">
    <p>Reuse an expensive shipping quote for a zone you already asked about.</p>
    <label>
      Shipping zone
      <select v-model="zone">
        <option>Europe</option>
        <option>Asia</option>
        <option>Americas</option>
      </select>
    </label>
    <div class="example-actions">
      <button type="button" @click="getQuote">Get quote</button>
      <button type="button" @click="() => quote.clear()">Forget quotes</button>
    </div>
    <output aria-live="polite"
      >{{ price === undefined ? "No quote yet." : `Shipping: €${price}` }} · Calculated
      {{ calculations }} times</output
    >
  </div>
</template>
