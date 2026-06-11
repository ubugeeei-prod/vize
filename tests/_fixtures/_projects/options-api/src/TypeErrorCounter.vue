<script lang="ts">
import { defineComponent } from "vue";

// Options API component with an INTENTIONAL type error: `Number.prototype.toFixed`
// takes an optional digit count (number), not a boolean. Both vize and vue-tsc
// must report TS2345 here, located inside this .vue file.
export default defineComponent({
  name: "TypeErrorCounter",
  data() {
    return {
      count: 0,
    };
  },
  computed: {
    formatted(): string {
      return this.count.toFixed(true);
    },
  },
});
</script>

<template>
  <span>{{ formatted }}</span>
</template>
