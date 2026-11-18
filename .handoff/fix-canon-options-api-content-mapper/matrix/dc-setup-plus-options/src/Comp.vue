<script lang="ts">
import { defineComponent, ref, computed } from 'vue'
export default defineComponent({
  props: { start: { type: Number, default: 0 } },
  setup(props) {
    const n = ref(props.start)
    const label = computed(() => String(n.value))
    function bump(by: number) { n.value += by }
    return { n, label, bump }
  },
  data() { return { extra: 'e' } },
  computed: { both(): string { return this.label + this.extra } },
  methods: { twice() { this.bump(2); return this.n * 2 } },
})
</script>

<template>
  <div>
    {{ n.toFixed() }} {{ label.toUpperCase() }} {{ both.length }} {{ twice() }} {{ start }}
    <button @click="bump(1)">+</button>
  </div>
</template>
