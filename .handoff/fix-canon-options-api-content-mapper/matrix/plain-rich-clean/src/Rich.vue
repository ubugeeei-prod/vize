<script lang="ts">
import { defineComponent, type PropType } from 'vue'
interface Item { id: number; name: string }
export default {
  props: {
    title: { type: String, required: true },
    count: { type: Number, default: 0 },
    items: { type: Array as PropType<Item[]>, default: () => [] },
    mode: { type: String as PropType<'a' | 'b'>, validator: (v: string) => ['a', 'b'].includes(v) },
    flag: Boolean,
    loose: null,
    multi: [String, Number],
    maybe: String,
  },
  emits: ['change'],
  data() {
    return { local: 1, list: [] as string[] }
  },
  computed: {
    upper(): string { return this.title.toUpperCase() },
    rw: {
      get(): number { return this.count },
      set(v: number) { this.local = v },
    },
  },
  methods: {
    inc(by: number) { this.local += by; this.$emit('change') },
  },
  watch: {
    count(n: number) { this.local = n },
  },
}
</script>

<template>
  <div :title="title.toUpperCase()">
    {{ count.toFixed(2) }} {{ items.map((i) => i.name).join() }} {{ mode }}
    {{ flag ? 'y' : 'n' }} {{ upper.length }} {{ rw.toFixed() }} {{ local + 1 }}
    {{ list.length }} {{ multi }} {{ loose }} {{ maybe?.length }}
    <button @click="inc(1)">+</button>
    <input v-model="rw" type="number" />
    <span v-for="item in items" :key="item.id">{{ item.name.toUpperCase() }}</span>
  </div>
</template>
