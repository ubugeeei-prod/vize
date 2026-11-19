<script setup>
import RouterLink from "./MyLink.vue";
</script>

<template>
  <!-- A local `RouterLink` is not Vue Router's: no claim. -->
  <RouterLink :to="{ name: 'nope' }">local link</RouterLink>
</template>
