<script setup lang="ts" vapor>
defineProps<{ label: string }>();
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
    <textarea>&lt;span&gt;</textarea>
  </section>
</template>

<style scoped>
section {
  color: red;
}
</style>
