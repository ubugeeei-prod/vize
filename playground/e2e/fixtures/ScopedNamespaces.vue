<script setup lang="ts" vapor>
const { label } = defineProps<{ label: string }>();
</script>

<template>
  <section>
    <svg>
      <g><circle r="2" /></g>
    </svg>
    <math
      ><mrow
        ><mi>{{ label }}</mi></mrow
      ></math
    >
    <textarea aria-label="Raw text sample">&lt;span&gt;</textarea>
  </section>
</template>

<style scoped>
section {
  color: red;
}
</style>
