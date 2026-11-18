<script lang="ts">
import { defineComponent } from 'vue'
import { counter, named, base } from './mixins'
export default defineComponent({
  mixins: [counter, named],
  extends: base,
})
</script>

<template>
  <div>
    {{ count.toUpperCase() }} {{ greet(1) }} {{ label.toFixed() }}
    {{ baseValue.toFixed() }} {{ baseMethod('x') }} {{ notInMixin }}
  </div>
</template>
