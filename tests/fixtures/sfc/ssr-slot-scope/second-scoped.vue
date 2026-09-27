<template>
  <div class="layout"><slot /></div>
</template>
<style scoped>
.layout {
  display: grid;
  gap: 2rem;
}
</style>
<style scoped>
:slotted(.title) {
  letter-spacing: 0.05em;
}
</style>
