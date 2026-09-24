<!-- Inline command palette that filters grouped actions as the user types and reports the last one run. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  CommandPalette,
  CommandPaletteEmpty,
  CommandPaletteGroup,
  CommandPaletteInput,
  CommandPaletteItem,
  CommandPaletteList,
} from "../command-palette.ts";

const lastAction = ref<string | null>(null);

function run(action: string | undefined): void {
  if (action !== undefined) lastAction.value = action;
}
</script>

<template>
  <div>
    <CommandPalette>
      <CommandPaletteInput aria-label="Search commands" placeholder="Type a command" />
      <CommandPaletteList aria-label="Commands">
        <CommandPaletteGroup heading="Files">
          <CommandPaletteItem
            value="new-file"
            text-value="New file"
            shortcut="Control+N"
            @select="run"
          />
          <CommandPaletteItem
            value="open-file"
            text-value="Open file"
            shortcut="Control+O"
            @select="run"
          />
        </CommandPaletteGroup>
        <CommandPaletteGroup heading="Preferences">
          <CommandPaletteItem
            value="toggle-theme"
            text-value="Toggle dark theme"
            :keywords="['appearance', 'color']"
            @select="run"
          />
          <CommandPaletteItem value="open-settings" text-value="Open settings" @select="run" />
        </CommandPaletteGroup>
        <CommandPaletteEmpty>No commands found</CommandPaletteEmpty>
      </CommandPaletteList>
    </CommandPalette>
    <output>Last command: {{ lastAction ?? "none" }}</output>
  </div>
</template>
