<template>
  <Layout viewMode="pc">
    <template #side="{ viewMode }">
      <Counter :count="viewMode" />
    </template>
  </Layout>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import Layout from './Layout.vue'

const Counter = defineComponent({
  name: 'Counter',
  props: { count: { type: Number, required: true } },
})

export default defineComponent({
  name: 'Wrong',
  components: { Layout, Counter },
})
</script>
