<script lang="ts">
import { defineComponent } from 'vue'
import Base from './Base.vue'
export default defineComponent({
  mixins: [Base],
})
</script>

<template>
  <div>{{ baseCount.toFixed() }} <button @click="baseAdd('x')">bad</button> {{ baseUpper.toFixed() }}</div>
</template>
