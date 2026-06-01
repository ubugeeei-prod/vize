<script setup lang="ts">
import { inject, provide, type InjectionKey } from "vue";

const countKey: InjectionKey<number> = Symbol("count");

provide(countKey, 1);

const count = inject(countKey, 0);
console.log(count.toFixed(0));
</script>

<template>
  <div>{{ count }}</div>
</template>
