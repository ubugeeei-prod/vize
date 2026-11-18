<script>
import { defineComponent } from 'vue'
export default defineComponent({
  data() {
    return {
      /** @type {string[]} */
      list: [],
      count: 0,
    }
  },
  methods: {
    /** @param {string} item */
    push(item) { this.list.push(item); this.count++ },
  },
})
</script>

<template>
  <div>
    {{ list.join() }} {{ count.toFixed() }}
    <button @click="push('a')">a</button>
    <button @click="push(1)">1</button>
  </div>
</template>
