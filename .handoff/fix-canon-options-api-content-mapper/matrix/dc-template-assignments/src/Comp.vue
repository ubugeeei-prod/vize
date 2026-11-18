<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: { type: Number, required: true } },
  data() { return { local: 0, text: '' } },
  computed: { ro(): number { return this.local } },
})
</script>

<template>
  <div>
    <button @click="local++">ok</button>
    <button @click="count++">prop</button>
    <button @click="ro = 2">computed</button>
    <button @click="text = 1">type</button>
    <input v-model="count" />
  </div>
</template>
