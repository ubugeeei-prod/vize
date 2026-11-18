<script lang="ts">
import { defineComponent } from 'vue'
import Icon from './Icon.vue'
export const shared = 1
export default defineComponent({
  components: { Icon },
})
</script>

<script setup lang="ts">
defineProps<{ name: string }>()
</script>

<template>
  <Icon :name="name" /> {{ shared }}
  <Icon :name="1" />
</template>
