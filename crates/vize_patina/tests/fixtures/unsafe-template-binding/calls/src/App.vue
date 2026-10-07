<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
import { PREFIX, isReady } from "./consts";

const count = ref(0);
const localOk = () => true;
const emit = defineEmits<{ play: [] }>();
</script>

<template>
  <div>
    <Child :disabled="!isReady()" />
    <Child :disabled="isReady()" />
    <Child :disabled="!localOk()" />
    <Child :disabled="localOk()" />
    <Child :label="PREFIX" />
    <Child @play="() => emit('play')" />
    <Child @play="emit('play')" />
    <Child @play="$emit('play')" />
    <Child @play="() => count++" />
  </div>
</template>
