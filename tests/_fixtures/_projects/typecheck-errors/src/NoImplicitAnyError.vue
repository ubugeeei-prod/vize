<script setup lang="ts">
// Regression for #966: TS7006 (implicit any) must surface from `vize check`
// when `noImplicitAny`/`strict` is on. Previously these codes were silently
// suppressed alongside virtual-TS-driven codes.
function handle(e) {
  console.log(e);
}
</script>

<template>
  <button @click="handle">Click</button>
</template>
