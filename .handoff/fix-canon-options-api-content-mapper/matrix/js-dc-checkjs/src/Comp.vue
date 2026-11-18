<script>
import { defineComponent } from 'vue'
export default defineComponent({
  props: {
    msg: { type: String, required: true },
    count: { type: Number, default: 0 },
  },
  data() {
    return { local: 1 }
  },
  computed: {
    /** @returns {string} */
    upper() { return this.msg.toUpperCase() },
  },
  methods: {
    /** @param {number} n */
    bump(n) { this.local += n },
  },
})
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ count.toFixed() }} {{ local }} {{ upper.length }}
    <button @click="bump(1)">+</button>
    <button @click="bump('x')">x</button>
    {{ msg.toFixed() }} {{ nope }}
  </div>
</template>
