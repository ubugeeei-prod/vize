<script lang="ts">
import { defineComponent, defineAsyncComponent } from 'vue'
export default defineComponent({
  components: { AsyncChild: defineAsyncComponent(() => import('./Child.vue')) },
})
</script>

<template>
  <AsyncChild title="a" />
  <AsyncChild :title="1" />
  <GlobalChild title="a" />
  <GlobalChild :title="1" />
  <global-child :title="2" />
</template>
