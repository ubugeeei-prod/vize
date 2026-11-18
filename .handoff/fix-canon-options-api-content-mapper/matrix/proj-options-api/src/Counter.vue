<script lang="ts">
import { defineComponent } from "vue";

// Options API component exercising props / data / computed / methods.
export default defineComponent({
  name: "Counter",
  props: {
    label: {
      type: String,
      required: true,
    },
    step: {
      type: Number,
      default: 1,
    },
  },
  data() {
    return {
      count: 0,
    };
  },
  computed: {
    // `this` is typed through the defineComponent instance bridge.
    doubled(): number {
      return this.count * 2;
    },
    heading(): string {
      return `${this.label}: ${this.count.toFixed(0)}`;
    },
  },
  methods: {
    increment(): void {
      this.count += this.step;
    },
  },
});
</script>

<template>
  <button type="button" @click="increment">
    {{ heading }} (x2 = {{ doubled }}, step {{ step }})
  </button>
</template>
