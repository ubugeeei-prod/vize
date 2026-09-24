<!-- Controlled bottom tab bar that switches between app sections and shows an unread badge. -->
<script setup lang="ts">
import { ref } from "vue";

import { BottomNavigation, BottomNavigationItem } from "../bottom-navigation.ts";

type Section = "home" | "search" | "inbox" | "profile";

const sections: readonly Section[] = ["home", "search", "inbox", "profile"];
const active = ref<Section>("home");
</script>

<template>
  <div>
    <output>Showing: {{ active }}</output>
    <BottomNavigation v-model="active" :destinations="sections" aria-label="Main sections">
      <BottomNavigationItem value="home">Home</BottomNavigationItem>
      <BottomNavigationItem value="search">Search</BottomNavigationItem>
      <BottomNavigationItem value="inbox" :badge="3">Inbox</BottomNavigationItem>
      <BottomNavigationItem value="profile">Profile</BottomNavigationItem>
    </BottomNavigation>
  </div>
</template>
