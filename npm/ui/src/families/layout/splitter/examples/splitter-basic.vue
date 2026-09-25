<!-- Two-pane horizontal splitter with a collapsible sidebar and a labelled resize handle. -->
<script setup lang="ts">
import { SplitterGroup, SplitterHandle, SplitterPanel } from "../splitter.ts";
</script>

<template>
  <SplitterGroup>
    <SplitterPanel :default-size="30" :min-size="20" collapsible>
      <nav aria-label="Folders">
        <ul>
          <li>Inbox</li>
          <li>Archive</li>
        </ul>
      </nav>
    </SplitterPanel>
    <SplitterHandle aria-label="Resize folder list" />
    <SplitterPanel :default-size="70" :min-size="40">
      <p>Select a folder to read its messages.</p>
    </SplitterPanel>
  </SplitterGroup>
</template>
