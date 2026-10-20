<script setup lang="ts">
import ItemListAlias from "#ui/item-list.vue";
import ItemListRelative from "../ui/item-list.vue";

const items = ["a", "b"];
</script>

<template>
  <ItemListAlias :items>
    <template #item="{ value }">{{ value.toUpperCase() }}</template>
  </ItemListAlias>
  <ItemListRelative :items>
    <template #item="{ value }">{{ value.toUpperCase() }}</template>
  </ItemListRelative>
</template>
