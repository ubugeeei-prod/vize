<script lang="ts">
import type { ComponentOptions } from 'vue'
const options: ComponentOptions = {
  data() { return { n: 1 } },
}
export default options
</script>

<template><div>{{ n }} {{ anything }}</div></template>
