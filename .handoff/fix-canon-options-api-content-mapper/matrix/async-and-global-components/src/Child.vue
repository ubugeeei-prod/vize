<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { title: { type: String, required: true } } })
</script>

<template><div>{{ title }}</div></template>
