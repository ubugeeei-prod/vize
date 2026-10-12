<script setup lang="ts">
const t = (id: string, params?: Record<string, unknown>): string => id;
const flag = true;
const name = "x";
</script>

<template>
  <!-- reported: the alternate holds an object literal, and the ternary spans lines -->
  <p>{{
    flag
      ? t("a")
      : t("b", { name })
  }}</p>
  <!-- not reported: the same expression on one line -->
  <p>{{ flag ? t("a") : t("b", { name }) }}</p>
  <!-- not reported: the object literal in the consequent -->
  <p>{{
    flag
      ? t("b", { name })
      : t("a")
  }}</p>
</template>
