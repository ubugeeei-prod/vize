<script setup lang="ts">
import { useToast } from "./useToast";

const { show } = useToast();
</script>

<template>
  <button @click="show('app')">App</button>
</template>
