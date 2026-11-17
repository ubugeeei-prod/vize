<script lang="ts">
import { defineComponent } from "vue";
import Child from "./Child.vue";

export default defineComponent({
  components: {
    Child,
    Aliased: Child,
  },
});
</script>

<template>
  <Child label="shorthand" />
  <Aliased label="aliased" />
</template>
