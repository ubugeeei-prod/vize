<!-- Application menubar with File and Edit menus whose open menu is bound with v-model. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarRoot,
  MenubarSeparator,
  MenubarTrigger,
} from "../menubar.ts";

const openMenu = ref<string | null>(null);
const lastCommand = ref("none");
</script>

<template>
  <div>
    <MenubarRoot v-model="openMenu" aria-label="Editor">
      <MenubarMenu value="file">
        <MenubarTrigger>File</MenubarTrigger>
        <MenubarContent>
          <MenubarItem @select="() => (lastCommand = 'New file')">New file</MenubarItem>
          <MenubarItem @select="() => (lastCommand = 'Open')">Open…</MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled>Export as PDF</MenubarItem>
        </MenubarContent>
      </MenubarMenu>
      <MenubarMenu value="edit">
        <MenubarTrigger>Edit</MenubarTrigger>
        <MenubarContent>
          <MenubarItem @select="() => (lastCommand = 'Undo')">Undo</MenubarItem>
          <MenubarItem @select="() => (lastCommand = 'Redo')">Redo</MenubarItem>
        </MenubarContent>
      </MenubarMenu>
    </MenubarRoot>
    <output>Last command: {{ lastCommand }}</output>
  </div>
</template>
