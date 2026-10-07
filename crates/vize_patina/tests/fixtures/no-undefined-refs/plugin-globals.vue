<script setup lang="ts">
const name = "Vue";
</script>

<template>
  <p :title="$t('hello')">{{ $t("greeting", { name }) }}</p>
  <p v-if="$vuetify.display.mobile">{{ $filters.upper(name) }}</p>
  <p>{{ $route.path }} {{ $attrs.id }} {{ $slots.default }} {{ $props }} {{ $el }}</p>
  <p>{{ $customPlugin.format(name) }} {{ missing }}</p>
</template>
