<script setup lang="ts">
import { ref } from "vue"
const café = ref("chaud")
defineProps<{ title: string }>()
</script>
<template><h1 :title="title">{{ café }}</h1></template>
