<!-- Destructive confirmation that only closes through its explicit Cancel or Delete actions. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "../alert-dialog.ts";

const open = ref(false);
const deleted = ref(false);
</script>

<template>
  <div>
    <AlertDialogRoot v-model:open="open">
      <AlertDialogTrigger>Delete project</AlertDialogTrigger>
      <AlertDialogPortal>
        <AlertDialogOverlay />
        <AlertDialogContent>
          <AlertDialogTitle>Delete project?</AlertDialogTitle>
          <AlertDialogDescription>
            This permanently removes the project and all of its deployments.
          </AlertDialogDescription>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction @click="() => (deleted = true)">Delete</AlertDialogAction>
        </AlertDialogContent>
      </AlertDialogPortal>
    </AlertDialogRoot>
    <output>{{ deleted ? "Project deleted" : "Project kept" }}</output>
  </div>
</template>
