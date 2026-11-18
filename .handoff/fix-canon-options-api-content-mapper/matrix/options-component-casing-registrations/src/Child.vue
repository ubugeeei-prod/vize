<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { n: { type: Number, required: true } } })
</script>

<template><div>{{ n }}</div></template>
