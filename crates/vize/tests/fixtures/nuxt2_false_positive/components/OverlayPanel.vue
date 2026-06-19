<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  emits: {
    "update:is-opened-overlay-loading": (_value: boolean) => true,
  },
  methods: {
    close() {
      this.$emit("update:is-opened-overlay-loading", false);
    },
  },
});
</script>

<template>
  <button @click="close">Close</button>
</template>
