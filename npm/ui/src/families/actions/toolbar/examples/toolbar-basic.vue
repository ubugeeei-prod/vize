<!-- Labelled editor toolbar with roving focus that reports which action was pressed. -->
<script setup lang="ts">
import { ref } from "vue";

import { Toolbar, ToolbarItem } from "../toolbar.ts";

const lastAction = ref("none");
</script>

<template>
  <div>
    <Toolbar aria-label="Document actions" @press="(value) => (lastAction = value)">
      <ToolbarItem value="save">Save</ToolbarItem>
      <ToolbarItem value="duplicate">Duplicate</ToolbarItem>
      <ToolbarItem value="publish" disabled>Publish</ToolbarItem>
    </Toolbar>
    <output>Last action: {{ lastAction }}</output>
  </div>
</template>
