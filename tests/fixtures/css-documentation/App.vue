<script setup lang="ts">
const theme = { color: "#663399" };
</script>

<template>
  <section class="demo"><span class="child">CSS documentation</span><slot /></section>
</template>

<style scoped>
.demo {
  display: flex;
  color: red;
  width: auto;
  --brand: #663399;
  background-color: var(--brand);
  border-color: v-bind("theme.color");

  &:hover {
    color: rebeccapurple;
  }
}

.demo::before {
  content: "CSS";
}

@media (min-width: 40rem) {
  .demo {
    display: grid;
  }
}

.demo :deep(.child) {
  color: blue;
}
:slotted(div) {
  margin: auto;
}
:global(.theme) {
  color: inherit;
}
</style>
