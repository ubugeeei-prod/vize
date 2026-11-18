<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
import UnusedChild from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { maybe: undefined as string | undefined } },
})
</script>

<template>
  <Child title="x" :opt="maybe" @change="() => {}" />
</template>
