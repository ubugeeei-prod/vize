<template>
  <Forwarder
    ><slot><b>fallback</b></slot></Forwarder
  >
</template>
<style scoped>
:slotted(.title) {
  color: blue;
}
</style>
