<script setup>
import Child from './Child.vue';
import { state, record } from './fixture';
</script>
<template>
  <section>
    <template v-if="state.ready">
      <Child v-for="item in state.items" :key="item.id" :data-id="item.id"
        :label="item.label" @example="record($event)">
        <template #default="{ label }"><span>{{ label }}:{{ state.label }}</span></template>
      </Child>
    </template>
    <p v-else>waiting</p>
    <output>{{ state.label }}</output>
  </section>
</template>
<style scoped>
span { color: red; }
</style>
