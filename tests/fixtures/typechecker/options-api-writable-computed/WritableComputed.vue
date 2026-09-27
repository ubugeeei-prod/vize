<template>
  <div>
    <input :value="ratio" @input="ratio = ($event.target as HTMLInputElement).value" />
    <button @click="readonlyLabel = 'x'">{{ readonlyLabel }}</button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'WritableComputed',
  data() {
    return { store: { ratio: '1' } }
  },
  computed: {
    ratio: {
      get(): string {
        return this.store.ratio
      },
      set(value: string) {
        this.store.ratio = value
      },
    },
    readonlyLabel(): string {
      return this.store.ratio
    },
  },
})
</script>
