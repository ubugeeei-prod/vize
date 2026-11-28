<script setup lang="ts">
import { ref } from "vue";
import NativeView from "./components/NativeView.vue";
import NativeText from "./components/NativeText.vue";
import NativeButton from "./components/NativeButton.vue";

const count = ref(0);
</script>

<template>
  <NativeView :gap="16" :padding="24">
    <NativeText :text="`Count: ${count}`" />
    <NativeButton label="Increment" @press="count++" />
  </NativeView>
</template>

<style scoped></style>
