<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { v: 'x' } },
  methods: { onChanged() {} },
})
</script>

<template>
  <Child @changed="onChanged" v-model:value="v" @unknown-event="onChanged" />
  <Child @changed="missingHandler" />
</template>
