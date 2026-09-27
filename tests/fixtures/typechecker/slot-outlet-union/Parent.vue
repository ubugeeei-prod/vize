<template>
  <Layout :viewMode="mode">
    <template #panel="{ viewMode }">
      <Preview :viewMode="viewMode" />
    </template>
    <template #side="{ viewMode }">
      <Preview :viewMode="viewMode" />
    </template>
    <template #item="{ viewMode, entry }">
      <Preview :viewMode="viewMode" />
      {{ entry?.toUpperCase() }}
    </template>
  </Layout>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import Layout from './Layout.vue'
import Preview from './Preview.vue'

export default defineComponent({
  name: 'Parent',
  components: { Layout, Preview },
  data() {
    return { mode: 'pc' as 'pc' | 'sp' }
  },
})
</script>
