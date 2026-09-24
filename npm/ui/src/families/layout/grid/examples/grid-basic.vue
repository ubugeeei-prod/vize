<!-- Three equal columns of pricing plans laid out with native CSS grid. -->
<script setup lang="ts">
import { Grid } from "../grid.ts";

const plans = [
  { name: "Starter", price: "$0" },
  { name: "Team", price: "$12" },
  { name: "Enterprise", price: "Contact us" },
];
</script>

<template>
  <Grid as="ul" :columns="3" :gap="16" align="start">
    <li v-for="plan in plans" :key="plan.name">
      <h3>{{ plan.name }}</h3>
      <p>{{ plan.price }}</p>
    </li>
  </Grid>
</template>
