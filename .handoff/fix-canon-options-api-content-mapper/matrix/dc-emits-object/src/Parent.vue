<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { text: 'a', num: 1 } },
  methods: {
    onSave(p: { id: number }) { return p.id },
    onBadSave(p: { id: string }) { return p.id },
  },
})
</script>

<template>
  <Child @save="onSave" @close="() => {}" v-model="text" />
  <Child @save="onBadSave" />
  <Child v-model="num" />
</template>
