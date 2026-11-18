<script lang="ts">
import { defineComponent } from 'vue'
import { mapState, mapActions, mapWritableState, mapStores } from 'pinia'
import { useCounter } from './store'
export default defineComponent({
  computed: {
    ...mapState(useCounter, ['count', 'double']),
    ...mapState(useCounter, { renamed: 'name', fn: (s) => s.count + 1 }),
    ...mapWritableState(useCounter, ['name']),
    ...mapStores(useCounter),
  },
  methods: { ...mapActions(useCounter, ['increment']) },
})
</script>

<template>
  <div>
    {{ count.toFixed() }} {{ double.toFixed() }} {{ renamed.toUpperCase() }} {{ fn.toFixed() }}
    {{ name }} {{ counterStore.count }}
    <button @click="increment(1)">+</button>
    <input v-model="name" />
  </div>
</template>
