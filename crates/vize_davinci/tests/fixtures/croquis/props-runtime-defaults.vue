<script setup lang="ts">
const props = defineProps({
  label: {
    type: String,
    default: "Untitled",
  },
  step: {
    type: Number,
    required: true,
  },
  tags: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(["step-change"]);

function bump() {
  emit("step-change", props.step + 1);
}
</script>

<template>
  <button @click="bump">{{ props.label }} ({{ props.tags.length }})</button>
</template>
