<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    tags: { type: Array as PropType<string[]>, default: () => [] },
    opt: String,
  },
  emits: ['change'],
  data() { return { map: {} as Record<string, number>, n: 0 } },
  computed: { first(): string | undefined { return this.tags[0] } },
  methods: {
    handle(e: Event, unusedParam: number) { return e.type },
    unusedMethod() { return 1 },
  },
})
</script>

<template>
  <div @click="handle($event, 1)">
    {{ title }} {{ first?.length }} {{ tags[0].length }} {{ map.key }} {{ map['key'] }} {{ opt }}
  </div>
</template>
