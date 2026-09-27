<template>
  <div>{{ viewMode }}</div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'Preview',
  props: { viewMode: { type: String as PropType<'pc' | 'sp'>, required: true } },
})
</script>
