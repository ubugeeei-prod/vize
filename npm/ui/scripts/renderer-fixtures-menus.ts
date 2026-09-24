/** Menu-family fixtures compiled by every supported renderer lane. */
export const menuRendererFixtures = [
  {
    filename: "MenuConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  MenuArrow,
  MenuCheckboxItem,
  MenuContent,
  MenuGroup,
  MenuItem,
  MenuItemIndicator,
  MenuLabel,
  MenuRadioGroup,
  MenuRadioItem,
  MenuRoot,
  MenuSeparator,
  MenuSub,
  MenuSubContent,
  MenuSubTrigger,
  MenuTrigger,
} from "./families/menus/menu/menu.ts";
import type { MenuSelectEvent } from "./families/menus/menu/menu.ts";

const wrap = ref(true);
const size = ref<"small" | "large" | null>("small");
function keepOpen(event: MenuSelectEvent): void {
  event.preventDefault();
}
</script>

<template>
  <MenuRoot id="renderer-menu" default-open>
    <MenuTrigger>Actions</MenuTrigger>
    <MenuContent portal-disabled>
      <MenuGroup>
        <MenuLabel>File</MenuLabel>
        <MenuItem text-value="New" @select="keepOpen">New</MenuItem>
        <MenuItem disabled>Delete</MenuItem>
      </MenuGroup>
      <MenuSeparator />
      <MenuCheckboxItem v-model="wrap">
        <MenuItemIndicator>✓</MenuItemIndicator>
        Wrap
      </MenuCheckboxItem>
      <MenuRadioGroup v-model="size">
        <MenuRadioItem value="small">Small</MenuRadioItem>
        <MenuRadioItem value="large">Large</MenuRadioItem>
      </MenuRadioGroup>
      <MenuSub>
        <MenuSubTrigger>Share</MenuSubTrigger>
        <MenuSubContent portal-disabled>
          <MenuItem>Email</MenuItem>
        </MenuSubContent>
      </MenuSub>
      <MenuArrow />
    </MenuContent>
  </MenuRoot>
</template>
`,
  },
  {
    filename: "DropdownMenuConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "./families/menus/dropdown-menu/dropdown-menu.ts";
</script>

<template>
  <DropdownMenuRoot id="renderer-dropdown" :modal="false">
    <DropdownMenuTrigger open-on="click">File</DropdownMenuTrigger>
    <DropdownMenuContent placement="bottom-end">
      <DropdownMenuItem>New</DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenuRoot>
</template>
`,
  },
  {
    filename: "ContextMenuConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "./families/menus/context-menu/context-menu.ts";
</script>

<template>
  <ContextMenuRoot id="renderer-context-menu">
    <ContextMenuTrigger :long-press-delay="500">Right-click the canvas</ContextMenuTrigger>
    <ContextMenuContent aria-label="Canvas actions" :offset="0">
      <ContextMenuItem>Paste</ContextMenuItem>
    </ContextMenuContent>
  </ContextMenuRoot>
</template>
`,
  },
  {
    filename: "MenubarConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarRoot,
  MenubarTrigger,
} from "./families/menus/menubar/menubar.ts";

const open = ref<string | null>(null);
</script>

<template>
  <MenubarRoot v-model="open" aria-label="Application">
    <MenubarMenu value="file">
      <MenubarTrigger>File</MenubarTrigger>
      <MenubarContent>
        <MenubarItem>New</MenubarItem>
      </MenubarContent>
    </MenubarMenu>
    <MenubarMenu value="edit">
      <MenubarTrigger>Edit</MenubarTrigger>
      <MenubarContent>
        <MenubarItem>Undo</MenubarItem>
      </MenubarContent>
    </MenubarMenu>
  </MenubarRoot>
</template>
`,
  },
] as const;
