<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { amount: 1 } },
  methods: { label() { return this.$t('x') + this.$filters.money(this.amount) } },
})
</script>

<template>
  <div>
    {{ $t('hello') }} {{ $filters.money(amount) }}
    {{ $t(1) }} {{ $filters.money('x') }} {{ $missingGlobal }}
  </div>
</template>
