<script setup lang="ts">
// Auto-imported types (no import statement): a user type, a generic user type,
// and Vue's `Ref` with both of its type parameters. Each must resolve to the
// real type, so this page type-checks clean.
const color: ColorName = "pink";
const box: Box<number> = { value: 1 };

function double(count: Ref<number, number>): number {
  return count.value * 2;
}

const counter = useCounter(2);
const doubled = double(counter.count);
</script>

<template>
  <p :class="color">{{ box.value }} / {{ doubled }}</p>
</template>
