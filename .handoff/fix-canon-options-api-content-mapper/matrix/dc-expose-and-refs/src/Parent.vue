<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  methods: {
    go() {
      const c = this.$refs.child as InstanceType<typeof Child>
      c.reset()
    },
  },
})
</script>

<template>
  <Child ref="child" />
  <input ref="input" />
  <button @click="go">go</button>
  {{ $refs.input }}
</template>
