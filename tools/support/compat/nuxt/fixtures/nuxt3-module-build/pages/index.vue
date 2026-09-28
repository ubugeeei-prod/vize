<script setup lang="ts">
const heading = "Nuxt 3 with Vize";
const cards = [
  { id: "alpha", label: "Alpha", featured: true },
  { id: "beta", label: "Beta", featured: false },
];
</script>

<template>
  <main>
    <h1>{{ heading }}</h1>
    <NuxtLink to="/about">About</NuxtLink>
    <ul data-test="cards">
      <li v-for="card in cards" :key="card.id">
        <BadgeCard :title="card.label" :featured="card.featured">
          <span class="card-id">{{ card.id }}</span>
        </BadgeCard>
      </li>
    </ul>
  </main>
</template>
