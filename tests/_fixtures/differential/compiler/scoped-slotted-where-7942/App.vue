<template>
  <div class="box"><slot /></div>
</template>

<style scoped>
:where(.box) {
  color: red;
}

:where(.box) > :slotted(:where(p)) {
  line-height: 1.1;
}
</style>
