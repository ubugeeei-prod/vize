export const commandPaletteRendererFixtures = [
  {
    filename: "CommandPaletteConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import { useCommandRouter } from "./families/foundations/command/command.ts";
import {
  CommandPaletteDialog,
  CommandPaletteEmpty,
  CommandPaletteGroup,
  CommandPaletteInput,
  CommandPaletteItem,
  CommandPaletteList,
  CommandPaletteRoot,
} from "./families/overlays/command-palette/command-palette.ts";

const router = useCommandRouter<"reload" | "theme">();
router.register({ id: "reload", title: "Reload window", run: () => undefined });
router.register({ id: "theme", title: "Toggle theme", keywords: ["dark"], run: () => undefined });
const search = ref("");
const recent = ref<readonly ("reload" | "theme")[]>([]);
</script>

<template>
  <CommandPaletteDialog shortcut="Mod+K">
    <CommandPaletteRoot v-slot="{ commands, resultCount }" v-model:search="search" v-model:recent="recent" :router>
      <CommandPaletteInput placeholder="Type a command" />
      <CommandPaletteList>
        <CommandPaletteGroup heading="Commands">
          <CommandPaletteItem v-for="command in commands" :key="command.id" :command="command.id" />
        </CommandPaletteGroup>
        <CommandPaletteEmpty>No results ({{ resultCount }})</CommandPaletteEmpty>
      </CommandPaletteList>
    </CommandPaletteRoot>
  </CommandPaletteDialog>
</template>
`,
  },
] as const;
