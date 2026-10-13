<script setup lang="ts">
type Props = {
  color: "primary" | "purple" | "orange" | "navy";
  title: string;
};

const props = defineProps<Props>();

const backgroundColor = `var(--color-${props.color}-base)`;
const textColor = `var(--color-${props.color}-sub)`;
</script>

<template>
  <th class="track">
    {{ title }}
  </th>
</template>

<style scoped>
.track {
  height: 80px;
  padding: 0 16px;
  color: v-bind(textColor);
  background-color: v-bind(backgroundColor);
  border-radius: 8px;
}
</style>
