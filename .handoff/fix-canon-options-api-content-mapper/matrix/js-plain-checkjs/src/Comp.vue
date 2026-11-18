<script>
export default {
  props: {
    msg: { type: String, required: true },
    count: { type: Number, default: 0 },
  },
  emits: ['done'],
  data() {
    return { local: 1, text: 'a' }
  },
  computed: {
    upper() { return this.msg.toUpperCase() },
  },
  methods: {
    bump(n) { this.local += n },
  },
}
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ count.toFixed() }} {{ local }} {{ upper.length }}
    <button @click="bump(1)">+</button>
    {{ msg.toFixed() }} {{ text.toFixed() }} {{ nope }}
  </div>
</template>
