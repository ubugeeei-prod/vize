<script lang="ts">
import BaseButton from './BaseButton.vue'

export default {
    name: 'Button',
    extends: BaseButton
}
</script>

<template>
<component v-if="!asChild" :is="as" :class="cx(1)">
  <span v-if="label">{{ label.toFixed() }}</span>
</component>
</template>
