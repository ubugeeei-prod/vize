<script setup lang="ts">
import { ref } from "vue";
import { Grid } from "./families/layout/grid/grid.ts";

const columns = ref(3);
</script>

<template>
  <Grid as="section" :columns gap="0.75rem" align="center" justify="stretch" auto-flow="row dense">
    <template #default="{ autoFlow, columns: resolvedColumns }">
      <article>{{ resolvedColumns }}</article>
      <article>{{ autoFlow }}</article>
    </template>
  </Grid>
</template>
