<script setup lang="ts">
type Handler = (value: string) => void;

const callbacks: Array<() => void> = [];
let handler: Handler | undefined;

function run(): void {
  for (const callback of callbacks) callback();
  handler?.("run");
}
</script>

<template>
  <button type="button" @click="run">Run</button>
</template>
