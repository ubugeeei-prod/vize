<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  computed: { id(): string { return String(this.$route.params.id) } },
  methods: { go() { this.$router.push('/') } },
})
</script>

<template>
  <div>
    {{ $route.path.toUpperCase() }} {{ id }} {{ $route.query.q }}
    <button @click="$router.push({ name: 'home' })">home</button>
    {{ $route.nope }} {{ $router.nope() }}
  </div>
</template>
