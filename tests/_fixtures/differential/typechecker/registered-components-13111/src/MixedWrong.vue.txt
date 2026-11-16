<script lang="ts">
import Child from "./Child.vue";

export default {
  components: { Aliased: Child },
};
</script>

<script setup lang="ts">
const count = 1;
</script>

<template>
  <Aliased :label="count" />
  <NotRegistered />
</template>
