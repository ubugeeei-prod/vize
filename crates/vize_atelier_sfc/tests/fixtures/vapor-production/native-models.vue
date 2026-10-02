<script setup>
import { state } from './fixture';
</script>
<template>
  <main>
    <input class="text" v-model.trim="state.value" />
    <input class="check" type="checkbox" v-model="state.enabled" true-value="yes" false-value="no" />
    <output>{{ state.value }}|{{ state.enabled }}</output>
  </main>
</template>
