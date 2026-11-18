<script lang="ts">
import { defineComponent } from 'vue'
import { Gen } from './Gen'
export default defineComponent({ components: { Gen } })
</script>

<template>
  <Gen :item="1" :list="[1, 2]" />
  <Gen item="a" :list="['b']" />
  <Gen :item="1" :list="['b']" />
</template>
