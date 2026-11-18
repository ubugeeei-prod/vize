<script lang="ts">
import { defineComponent, type Directive } from 'vue'
import Child from './Child.vue'
const vColor: Directive<HTMLElement, string> = { mounted(el, binding) { el.style.color = binding.value } }
export default defineComponent({
  components: { Child, Renamed: Child },
  directives: {
    focus: { mounted(el: HTMLElement) { el.focus() } },
    color: vColor,
  },
  data() { return { c: 'red', n: 1 } },
})
</script>

<template>
  <Child title="a" />
  <Renamed title="b" />
  <renamed :title="1" />
  <input v-focus v-color="c" />
  <input v-color="n" />
</template>
