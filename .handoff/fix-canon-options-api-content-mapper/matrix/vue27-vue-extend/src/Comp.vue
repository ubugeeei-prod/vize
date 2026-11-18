<script lang="ts">
import Vue from 'vue'
export default Vue.extend({
  props: { title: { type: String, required: true } },
  data() { return { local: 1 } },
  methods: { inc(by: number) { this.local += by } },
})
</script>

<template>
  <div>
    {{ title.toUpperCase() }} {{ local.toFixed() }}
    <button @click="inc(1)">+</button>
    {{ title.toFixed() }} {{ nope }}
  </div>
</template>
