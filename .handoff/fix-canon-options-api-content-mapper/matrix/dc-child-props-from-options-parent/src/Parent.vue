<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { n: 1, s: 'x' } },
  methods: { onPick(n: number) { this.n = n } },
})
</script>

<template>
  <Child title="ok" :size="n" kind="b" :tags="[s]" @pick="onPick" />
  <Child :title="n" />
  <Child />
  <Child title="x" kind="c" />
  <Child title="x" :tags="[n]" />
  <Child title="x" @pick="(v: string) => v" />
</template>
