<script setup lang="ts">
import { setupPanel } from "#ui/side-panel.vue";
import { setupPanel as setupPanelRelative } from "../ui/side-panel.vue";
import { label } from "#ui/helpers";

const viaAlias = setupPanel();
const viaRelative = setupPanelRelative();
</script>

<template>
  <button type="button" @click="viaAlias.open">alias</button>
  <button type="button" @click="viaRelative.open">relative</button>
  <span>{{ label() }}</span>
</template>
