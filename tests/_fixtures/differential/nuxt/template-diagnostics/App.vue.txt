<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
const onSelect = () => {};
function format() {}
class Store {}
const store = new Store();
const items = [1, 2];
</script>

<template>
  <main>
    <NuxtLink to="/missing">Missing</NuxtLink>
    <nuxt-link href="/about?tab=1">About</nuxt-link>
    <NuxtLink to="/docs" external>External</NuxtLink>
    <NuxtLink :to="'/dynamic'">Dynamic</NuxtLink>
    <NuxtLink to="/styled" :class="'active'">Styled</NuxtLink>
    <ServerCard
      v-model="count"
      @click.stop="onSelect"
      @error="onSelect"
      :format="format"
      :store="store"
      :key="onSelect"
      :label="'plain'"
      :render="() => 1"
    />
    <server-card :kind="Store" :pick="{ handler: onSelect }" :maybe="count ? onSelect : null" />
    <ul>
      <li v-for="(onSelect, index) in items" :key="index">
        <ServerCard :pick="onSelect" />
      </li>
    </ul>
    <NuxtIsland name="Chart" :props="{ render: () => 1, title: 'x', store: new Store(), [index]: format }" />
    <nuxt-island :context="{ 'quoted-key'() {} }" />
    <div v-pre><NuxtLink to="/raw">Raw</NuxtLink></div>
    <p>日本語 <NuxtLink to="/ja">ja</NuxtLink></p>
    <component is="vue:NuxtLink" to="/cast">Cast</component>
  </main>
</template>
