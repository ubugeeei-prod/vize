<script setup lang="ts">
function greet(name: string, age: number): string {
  return `${name} is ${age}`;
}
const result = greet(42, "not a number");
console.log(result);
</script>

<template>
  <div>{{ result }}</div>
</template>
