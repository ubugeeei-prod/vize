<script setup lang="ts">
import InfoCard from "./components/InfoCard.vue";
import PriceRow from "./components/PriceRow.vue";
import SafeBadge from "./components/SafeBadge.vue";
import LinkButton from "./components/LinkButton.vue";

const rows = [{ name: "Vize", price: "Free" }];
</script>

<template>
  <article>
    <p>
      Every template here is valid on its own.
      <InfoCard title="Composed" />
      <SafeBadge label="phrasing is fine" />
    </p>
    <table>
      <PriceRow v-for="row in rows" :key="row.name" :row="row" />
    </table>
    <a href="/docs"> Read the docs <LinkButton /> </a>
  </article>
</template>
