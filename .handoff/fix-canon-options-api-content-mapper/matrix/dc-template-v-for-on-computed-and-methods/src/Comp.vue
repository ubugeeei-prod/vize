<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { rows: [{ id: 1, name: 'a' }] } },
  computed: { sorted(): { id: number; name: string }[] { return [...this.rows] } },
  methods: { fmt(r: { id: number }) { return r.id.toFixed() } },
})
</script>

<template>
  <ul>
    <li v-for="r in sorted" :key="r.id">{{ fmt(r) }} {{ r.name.toUpperCase() }} {{ r.missing }}</li>
    <li v-for="(r, i) in rows" :key="i">{{ i.toFixed() }} {{ fmt({ id: 'x' }) }}</li>
  </ul>
</template>
