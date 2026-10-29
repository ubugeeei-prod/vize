<script lang="ts">
export default {
  props: {
    label: {
      type: String,
      required: true,
    },
    initial: {
      type: Number,
      default: 1,
    },
  },
  data() {
    return { count: this.initial };
  },
  computed: {
    doubled() {
      return this.count * 2;
    },
  },
  methods: {
    increment(step: number) {
      this.count += step;
      return this.count;
    },
  },
};
</script>

<template>
  {{ label }}:{{ count.toFixed(0) }}:{{ doubled.toFixed(0) }}:{{ increment(1).toFixed(0) }}
</template>
