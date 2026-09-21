<template>
  <div class="app">
    <header>
      <h1>{{ title }}</h1>
      <nav>
        <a v-for="link in links" :key="link.id" :href="link.url">{{ link.text }}</a>
      </nav>
    </header>
    <main>
      <section v-if="loading">Loading...</section>
      <section v-else>
        <article v-for="item in items" :key="item.id">
          <h2>{{ item.title }}</h2>
          <p>{{ item.body }}</p>
          <button @click="selectItem(item)">Select</button>
        </article>
      </section>
    </main>
    <footer><p>&copy; {{ year }}</p></footer>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
const title = ref('My App')
const loading = ref(false)
const items = ref([])
const links = ref([{ id: 1, url: '/', text: 'Home' }, { id: 2, url: '/about', text: 'About' }])
const year = computed(() => new Date().getFullYear())
function selectItem(item) { console.log('Selected:', item) }
</script>

<style scoped>
.app { max-width: 1200px; margin: 0 auto; }
header { display: flex; justify-content: space-between; }
</style>
