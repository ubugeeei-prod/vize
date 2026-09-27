<template>
  <div>
    <template v-if="isPC">
      <slot name="panel" :viewMode="viewMode" />
      <slot name="side" :viewMode="viewMode" />
    </template>
    <template v-else>
      <slot name="panel" />
      <slot name="side" viewMode="sp" />
    </template>
    <ul>
      <li v-for="entry in entries" :key="entry">
        <slot name="item" :viewMode="viewMode" :entry="entry" />
      </li>
    </ul>
    <slot name="item" :viewMode="viewMode" />
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'Layout',
  props: {
    viewMode: { type: String as PropType<'pc' | 'sp'>, required: true },
    entries: { type: Array as PropType<string[]>, default: () => [] },
  },
  computed: {
    isPC(): boolean {
      return this.viewMode === 'pc'
    },
  },
})
</script>
