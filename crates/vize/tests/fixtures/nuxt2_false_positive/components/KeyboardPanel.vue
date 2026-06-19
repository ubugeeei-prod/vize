<script lang="ts">
import { defineComponent, type PropType } from "vue";

export type Folder = { id: string; label: string };

export default defineComponent({
  props: {
    folders: {
      type: Array as PropType<Folder[]>,
      required: true,
    },
  },
  emits: {
    "click-folder": (_folder: Folder) => true,
    "input:math-key": (_key: string) => true,
  },
  methods: {
    clickFirst() {
      this.$emit("click-folder", this.folders[0]);
    },
    inputKey() {
      this.$emit("input:math-key", "plus");
    },
  },
});
</script>

<template>
  <button @click="clickFirst">{{ folders[0].label }}</button>
</template>
