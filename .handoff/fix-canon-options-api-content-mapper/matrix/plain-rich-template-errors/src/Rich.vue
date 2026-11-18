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
  <div>
    {{ title.toFixed() }}
    {{ count.toUpperCase() }}
    {{ unknownName }}
    <button @click="inc('x')">+</button>
    {{ items[0]!.nope }}
    {{ upper.toFixed() }}
    {{ local.length }}
    {{ maybe.length }}
    {{ mode === 'c' }}
  </div>
</template>
