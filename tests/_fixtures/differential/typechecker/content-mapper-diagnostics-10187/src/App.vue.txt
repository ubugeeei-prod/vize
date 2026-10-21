<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
import ModelChild from "./ModelChild.vue";

const text = ref("hello");
</script>

<template>
  <Child :count="'not a number'" />
  <ModelChild v-model="text" />
  <Child label="no count" />
</template>
