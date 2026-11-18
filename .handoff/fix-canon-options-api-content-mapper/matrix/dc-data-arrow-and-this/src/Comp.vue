<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { seed: { type: Number, required: true } },
  data: () => ({ a: 1, b: 'x' as string | null }),
  computed: {
    fromSeed(): number { return this.seed + this.a },
  },
  methods: {
    async load(): Promise<number> { return this.a },
  },
})
</script>

<template>
  <div @click="load">
    {{ a.toFixed() }} {{ b?.length }} {{ b.length }} {{ fromSeed }}
    <span v-if="b">{{ b.toUpperCase() }}</span>
  </div>
</template>
