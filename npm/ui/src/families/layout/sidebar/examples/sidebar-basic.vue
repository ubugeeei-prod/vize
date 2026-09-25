<!-- Application shell with a collapsible navigation sidebar, a rail, and a trigger in the main area. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from "../sidebar.ts";

const open = ref(true);
</script>

<template>
  <SidebarProvider v-model:open="open">
    <Sidebar aria-label="Mail navigation">
      <SidebarHeader>Acme Mail</SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Mailboxes</SidebarGroupLabel>
          <nav aria-label="Mailboxes">
            <ul>
              <li><a href="/mail/inbox" aria-current="page">Inbox</a></li>
              <li><a href="/mail/drafts">Drafts</a></li>
              <li><a href="/mail/sent">Sent</a></li>
              <li><a href="/mail/archive">Archive</a></li>
            </ul>
          </nav>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>Signed in as ada@example.com</SidebarFooter>
      <SidebarRail />
    </Sidebar>
    <SidebarInset>
      <SidebarTrigger />
      <h1>Inbox</h1>
      <p>The sidebar is {{ open ? "expanded" : "collapsed" }}.</p>
    </SidebarInset>
  </SidebarProvider>
</template>
