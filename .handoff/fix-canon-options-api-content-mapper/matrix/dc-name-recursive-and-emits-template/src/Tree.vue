<script lang="ts">
import { defineComponent, type PropType } from 'vue'
interface Node { label: string; children?: Node[] }
export default defineComponent({
  name: 'Tree',
  props: { node: { type: Object as PropType<Node>, required: true } },
  emits: ['select'],
})
</script>

<template>
  <div @click="$emit('select', node)">
    {{ node.label }}
    <Tree v-for="c in node.children" :key="c.label" :node="c" @select="$emit('select', $event)" />
  </div>
</template>
