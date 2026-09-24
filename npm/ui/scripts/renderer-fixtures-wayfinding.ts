export const wayfindingRendererFixtures = [
  {
    filename: "NavigationMenuConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuRoot,
  NavigationMenuTrigger,
  NavigationMenuViewport,
} from "./families/navigation/navigation-menu/navigation-menu.ts";

const open = ref<string | null>(null);
</script>

<template>
  <NavigationMenuRoot v-model="open" aria-label="Main" :delay-duration="150">
    <NavigationMenuList>
      <NavigationMenuItem value="products">
        <NavigationMenuTrigger>
          <template #default="{ state }">Products {{ state }}</template>
        </NavigationMenuTrigger>
        <NavigationMenuContent>
          <NavigationMenuLink href="/ui" active>UI</NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem value="blog">
        <NavigationMenuLink href="/blog">Blog</NavigationMenuLink>
      </NavigationMenuItem>
      <NavigationMenuIndicator />
    </NavigationMenuList>
    <NavigationMenuViewport />
  </NavigationMenuRoot>
</template>
`,
  },
  {
    filename: "ScrollSpyConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useScrollSpy } from "./families/interaction/scroll-spy/scroll-spy.ts";

const spy = useScrollSpy({ ids: ["intro", "usage"], offset: 64, initialActiveId: "intro" });
</script>

<template>
  <nav aria-label="Sections">
    <a href="#intro" :aria-current="spy.activeId.value === 'intro' ? 'location' : undefined">Intro</a>
    <a href="#usage" :aria-current="spy.activeId.value === 'usage' ? 'location' : undefined">Usage</a>
  </nav>
</template>
`,
  },
  {
    filename: "TimelineConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  TimelineConnector,
  TimelineContent,
  TimelineIndicator,
  TimelineItem,
  TimelineRoot,
  TimelineTime,
} from "./families/data/timeline/timeline.ts";

const steps = ["ordered", "shipped", "delivered"] as const;
</script>

<template>
  <TimelineRoot aria-label="Order" value="shipped">
    <TimelineItem v-for="step in steps" :key="step" :value="step">
      <TimelineIndicator />
      <TimelineConnector />
      <TimelineContent>
        <template #default="{ status }">{{ step }} {{ status }}</template>
      </TimelineContent>
      <TimelineTime datetime="2026-09-25">Sep 25</TimelineTime>
    </TimelineItem>
  </TimelineRoot>
</template>
`,
  },
  {
    filename: "TocConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { TocItem, TocLink, TocList, TocRoot } from "./families/navigation/toc/toc.ts";

const entries = [
  { id: "install", text: "Install" },
  { id: "usage", text: "Usage" },
] as const;
</script>

<template>
  <TocRoot default-active-id="install" :offset="64" scroll-behavior="smooth">
    <TocList>
      <TocItem v-for="entry in entries" :key="entry.id" :target-id="entry.id">
        <TocLink :target-id="entry.id">
          <template #default="{ active }">{{ entry.text }} {{ active }}</template>
        </TocLink>
      </TocItem>
    </TocList>
  </TocRoot>
</template>
`,
  },
] as const;
