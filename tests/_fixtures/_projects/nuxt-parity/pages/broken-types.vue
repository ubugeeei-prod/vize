<script setup lang="ts">
// INTENTIONAL type errors: every annotation below is an auto-imported type, so
// each wrong value must be caught (TS2322). A type that resolved to `any`, or
// did not resolve at all, would report something else or nothing.
const color: ColorName = "green";
const box: Box<number> = { value: "one" };
const counter = useCounter(2);
const label: Ref<string> = counter.count;
</script>

<template>
  <p :class="color">{{ box.value }} / {{ label }}</p>
</template>
