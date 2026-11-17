<script lang="ts">
import Child from "./Child.vue";

export default {
  components: { Aliased: Child },
};
</script>

<script setup lang="ts">
const label = "setup";
</script>

<template>
  <Aliased :label="label" />
</template>
