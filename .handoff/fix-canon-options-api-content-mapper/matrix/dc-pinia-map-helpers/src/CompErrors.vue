<script lang="ts">
import { defineComponent } from 'vue'
import { mapState, mapActions } from 'pinia'
import { useCounter } from './store'
export default defineComponent({
  computed: { ...mapState(useCounter, ['count']) },
  methods: { ...mapActions(useCounter, ['increment']) },
})
</script>

<template>
  <div>
    {{ count.toUpperCase() }}
    <button @click="increment('x')">+</button>
    {{ double }}
  </div>
</template>
