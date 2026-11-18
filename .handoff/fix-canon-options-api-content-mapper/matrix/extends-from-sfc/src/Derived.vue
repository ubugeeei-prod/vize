<script lang="ts">
import { defineComponent } from 'vue'
import Base from './Base.vue'
export default defineComponent({
  extends: Base,
  data() { return { own: 1 } },
})
</script>

<template>
  <div>
    {{ label.toUpperCase() }} {{ baseCount.toFixed() }} {{ baseUpper.length }} {{ own }}
    <button @click="baseAdd(1)">ok</button>
    <button @click="baseAdd('x')">bad</button>
    {{ label.toFixed() }} {{ baseCount.toUpperCase() }} {{ nope }}
  </div>
</template>
