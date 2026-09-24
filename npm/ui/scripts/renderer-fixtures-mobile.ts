export const mobileRendererFixtures = [
  {
    filename: "SafeAreaConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { SafeArea } from "./families/layout/safe-area/safe-area.ts";
</script>

<template>
  <SafeArea as="footer" :edges="['bottom', 'left', 'right']" apply="padding">
    <template #default="{ insets }">
      <output>{{ insets.bottom }}</output>
    </template>
  </SafeArea>
</template>
`,
  },
  {
    filename: "BottomNavigationConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  BottomNavigation,
  BottomNavigationItem,
} from "./families/navigation/bottom-navigation/bottom-navigation.ts";

const tab = ref<"home" | "inbox">("home");
</script>

<template>
  <BottomNavigation v-model="tab" :destinations="['home', 'inbox']" aria-label="Main">
    <BottomNavigationItem value="home" href="/">Home</BottomNavigationItem>
    <BottomNavigationItem value="inbox" :badge="3" v-slot="{ active }">
      {{ active ? "Inbox (open)" : "Inbox" }}
    </BottomNavigationItem>
  </BottomNavigation>
</template>
`,
  },
  {
    filename: "ActionSheetConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  ActionSheet,
  ActionSheetCancel,
  ActionSheetContent,
  ActionSheetItem,
  ActionSheetMenu,
  ActionSheetTitle,
  ActionSheetTrigger,
} from "./families/overlays/action-sheet/action-sheet.ts";

function onSelect(value: string): void {
  void value;
}
</script>

<template>
  <ActionSheet>
    <ActionSheetTrigger>Options</ActionSheetTrigger>
    <ActionSheetContent>
      <ActionSheetTitle>Photo</ActionSheetTitle>
      <ActionSheetMenu>
        <ActionSheetItem value="share" @select="(event) => onSelect(event.value)">Share</ActionSheetItem>
        <ActionSheetItem value="delete" destructive>Delete</ActionSheetItem>
      </ActionSheetMenu>
      <ActionSheetCancel>Cancel</ActionSheetCancel>
    </ActionSheetContent>
  </ActionSheet>
</template>
`,
  },
  {
    filename: "PullToRefreshConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  PullToRefresh,
  PullToRefreshTrigger,
} from "./families/interaction/pull-to-refresh/pull-to-refresh.ts";

const items = ref(["a", "b"]);
async function reload(): Promise<void> {
  items.value = [...items.value, "c"];
}
</script>

<template>
  <PullToRefresh :refresh-action="reload" :threshold="72">
    <template #default="{ state, progress }">
      <div :data-progress="progress">{{ state }}</div>
      <PullToRefreshTrigger v-slot="{ refreshing }">{{ refreshing ? "Refreshing" : "Refresh" }}</PullToRefreshTrigger>
      <ul><li v-for="item in items" :key="item">{{ item }}</li></ul>
    </template>
  </PullToRefresh>
</template>
`,
  },
  {
    filename: "SwipeActionsConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  SwipeActions,
  SwipeActionsAction,
  SwipeActionsContent,
  SwipeActionsTray,
} from "./families/interaction/swipe-actions/swipe-actions.ts";
import type { SwipeActionsOpen } from "./families/interaction/swipe-actions/swipe-actions.ts";

const open = ref<SwipeActionsOpen>(null);
</script>

<template>
  <ul>
    <SwipeActions v-model:open="open" as="li" @full-swipe="(side) => side">
      <SwipeActionsTray side="leading"><SwipeActionsAction value="pin">Pin</SwipeActionsAction></SwipeActionsTray>
      <SwipeActionsTray side="trailing"><SwipeActionsAction value="delete">Delete</SwipeActionsAction></SwipeActionsTray>
      <SwipeActionsContent v-slot="{ open: revealed }">{{ revealed ? "Actions shown" : "Message" }}</SwipeActionsContent>
    </SwipeActions>
  </ul>
</template>
`,
  },
  {
    filename: "PagerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  Pager,
  PagerPage,
  PagerTab,
  PagerTabList,
  PagerViewport,
} from "./families/navigation/pager/pager.ts";

const pages = ["all", "unread"] as const;
const page = ref<"all" | "unread">("all");
</script>

<template>
  <Pager v-model="page" :pages="pages">
    <template #default="{ index, count }">
      <PagerTabList aria-label="Mailboxes">
        <PagerTab v-for="item in pages" :key="item" :page="item">{{ item }}</PagerTab>
      </PagerTabList>
      <PagerViewport>
        <PagerPage v-for="item in pages" :key="item" :page="item" v-slot="{ active }">{{ active ? item : "" }}</PagerPage>
      </PagerViewport>
      <output>{{ index + 1 }} / {{ count }}</output>
    </template>
  </Pager>
</template>
`,
  },
] as const;
