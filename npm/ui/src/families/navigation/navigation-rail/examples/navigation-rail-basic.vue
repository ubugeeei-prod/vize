<!-- Collapsible side rail with three destinations and an unread badge. -->
<script setup lang="ts">
import { ref } from "vue";

import { NavigationRail, NavigationRailItem, NavigationRailToggle } from "../navigation-rail.ts";

const destinations = ["home", "search", "inbox"] as const;
const current = ref<(typeof destinations)[number]>("home");
const expanded = ref(false);
</script>

<template>
  <NavigationRail v-model="current" v-model:expanded="expanded" :destinations>
    <NavigationRailToggle aria-label="Toggle navigation labels">
      {{ expanded ? "«" : "»" }}
    </NavigationRailToggle>
    <NavigationRailItem value="home">Home</NavigationRailItem>
    <NavigationRailItem value="search">Search</NavigationRailItem>
    <NavigationRailItem value="inbox" :badge="3">Inbox</NavigationRailItem>
  </NavigationRail>
</template>
