<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    optional: String,
    withDefault: { type: String, default: 'x' },
    obj: { type: Object as PropType<{ a: number }> },
    req: { type: Object as PropType<{ a: number }>, required: true },
    fn: { type: Function as PropType<(n: number) => string>, required: true },
    date: { type: Date },
    union: { type: [String, Number] as PropType<string | number>, required: true },
  },
})
</script>

<template>
  <div>
    {{ optional.length }} {{ withDefault.length }} {{ obj.a }} {{ req.a.toFixed() }}
    {{ fn(1).length }} {{ fn('x') }} {{ date.getTime() }} {{ req.b }}
    <template v-if="typeof union === 'string'">{{ union.toUpperCase() }}</template>
    {{ union.toUpperCase() }}
  </div>
</template>
