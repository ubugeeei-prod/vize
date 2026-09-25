export const overlay3bRendererFixtures = [
  {
    filename: "SidebarConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import {
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarProvider,
  SidebarRail,
  SidebarRoot,
  SidebarTrigger,
} from "./families/layout/sidebar/sidebar.ts";

const open = ref(true);
</script>

<template>
  <SidebarProvider v-model:open="open" collapsible="icon" keyboard-shortcut="Mod+B">
    <SidebarRoot aria-label="Primary">
      <SidebarHeader>Workspace</SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Navigate</SidebarGroupLabel>
          <a href="/inbox">Inbox</a>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>Account</SidebarFooter>
      <SidebarRail />
    </SidebarRoot>
    <SidebarInset>
      <SidebarTrigger>Toggle</SidebarTrigger>
    </SidebarInset>
  </SidebarProvider>
</template>
`,
  },
  {
    filename: "ResizableConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import { ResizableHandle, ResizableRoot } from "./families/layout/resizable/resizable.ts";
import type { ResizableSize } from "./families/layout/resizable/resizable.ts";

const size = ref<ResizableSize>({ width: 320, height: 240 });
</script>

<template>
  <ResizableRoot v-model:size="size" :min-width="120" :max-width="640" lock-aspect-ratio>
    Panel
    <ResizableHandle edge="end" />
    <ResizableHandle edge="se" />
  </ResizableRoot>
</template>
`,
  },
] as const;
