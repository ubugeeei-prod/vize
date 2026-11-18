<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inheritAttrs: false,
  props: { title: { type: String, required: true } },
  emits: { pick: (n: number) => true },
})
</script>

<template>
  <div v-bind="$attrs">
    {{ $props.title.toUpperCase() }}
    {{ $attrs.class }}
    <slot v-if="$slots.default" />
    {{ $el }}
    {{ $refs }}
    {{ $options.name }}
    {{ $parent }} {{ $root }} {{ $data }}
    <button @click="$emit('pick', 1)">p</button>
    <button @click="$emit('pick', 'x')">p</button>
    {{ $props.nope }}
    {{ $nextTick }} {{ $forceUpdate }} {{ $watch }}
  </div>
</template>
