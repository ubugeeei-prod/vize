<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
import type TypeOnly from "./Child.vue";

const label = ref("a");
const items = ref([{ id: 1, name: "x" }]);
const childRef = ref<InstanceType<typeof Child>>();
const unused = 1;
function onChange(value: string) {
  label.value = value;
}
</script>

<template>
  <Child ref="childRef" v-model:label="label" :count="'not a number'" @change="onChange">
    <template #footer="{ total }">{{ total.toFixed(1) }}{{ total.missing }}</template>
    <template #header>no such slot</template>
  </Child>
  <Child
    v-for="item in items"
    :key="item.id"
    :count="item.name"
    @update:label="(value: number) => value"
  />
  <TypeOnly :count="1" />
  <Child v-if="label" :count="items.length" :label="label.missing" />
</template>
