<!-- Top app bar with a menu button, a title, and a search action that hides while scrolling down. -->
<script setup lang="ts">
import { AppBar } from "../app-bar.ts";
</script>

<template>
  <AppBar hide-on-scroll aria-labelledby="app-bar-title">
    <template #start>
      <button type="button" aria-label="Open navigation">≡</button>
    </template>
    <template #default="{ elevated }">
      <span id="app-bar-title" :data-elevated="elevated ? 'true' : undefined">Inbox</span>
    </template>
    <template #end>
      <button type="button">Search</button>
    </template>
  </AppBar>
</template>
