<!-- Empty project list that explains the situation and offers the next action. -->
<script setup lang="ts">
import { ref } from "vue";

import { EmptyState } from "../empty-state.ts";

const projects = ref<string[]>([]);

function createProject(): void {
  projects.value = [...projects.value, `Project ${projects.value.length + 1}`];
}
</script>

<template>
  <ul v-if="projects.length > 0">
    <li v-for="project in projects" :key="project">{{ project }}</li>
  </ul>
  <EmptyState v-else tone="neutral" density="comfortable">
    <h2>No projects yet</h2>
    <p>Projects group your documents and collaborators in one place.</p>
    <button type="button" @click="createProject">Create project</button>
  </EmptyState>
</template>
