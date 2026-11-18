<script lang="ts">
import { defineComponent } from 'vue'
import List from './List.vue'
export default defineComponent({ components: { List } })
</script>

<template>
  <List :items="['a']" v-slot="{ item, index }">{{ item.toUpperCase() }} {{ index.toFixed() }}</List>
  <List :items="['a']" v-slot="{ item }">{{ item.toFixed() }}</List>
  <List :items="[1]" />
</template>
