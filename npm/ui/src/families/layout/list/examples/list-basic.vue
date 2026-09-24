<!-- Unordered checklist and an ordered set of steps rendered as native lists. -->
<script setup lang="ts">
import { List } from "../list.ts";

const steps = ["Create a workspace", "Invite your team", "Connect a repository"];
</script>

<template>
  <section>
    <h3>What's included</h3>
    <List marker="disc" spacing="compact">
      <li>Unlimited projects</li>
      <li>Daily backups</li>
      <li>Email support</li>
    </List>
    <h3>Getting started</h3>
    <List as="ol" marker="decimal">
      <li v-for="step in steps" :key="step">{{ step }}</li>
    </List>
  </section>
</template>
