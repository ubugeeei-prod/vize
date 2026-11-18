<script lang="ts">
import { defineComponent } from 'vue'
import { counter, named, base } from './mixins'
export default defineComponent({
  mixins: [counter, named],
  extends: base,
  data() { return { own: true } },
})
</script>

<template>
  <div>
    {{ count.toFixed() }} {{ double }} {{ label.toUpperCase() }} {{ own }}
    <button @click="increment">+</button>
    {{ greet('x') }} {{ baseValue.toUpperCase() }} {{ baseMethod(1) }}
  </div>
</template>
