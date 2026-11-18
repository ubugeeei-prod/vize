<script lang="ts">
import { defineComponent } from 'vue'
import BaseButton from './BaseButton.vue'

export default defineComponent({
    name: 'Button',
    mixins: [BaseButton]
})
</script>

<template>
<button v-if="!asChild" :class="cx(2)">
  <span v-if="label">{{ label.toFixed() }}</span>
</button>
</template>
