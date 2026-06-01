<script setup lang="ts">
import { inject, type InjectionKey } from "vue";

const countKey: InjectionKey<number> = Symbol("count");
const count: string = inject(countKey, 0);

console.log(count);
</script>

<template>
  <div>{{ count }}</div>
</template>
