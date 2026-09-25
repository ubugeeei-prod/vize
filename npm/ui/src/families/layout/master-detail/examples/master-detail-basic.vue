<!-- Mailbox list with a detail pane side by side on wide screens and one pane at a time on narrow ones. -->
<script setup lang="ts">
import { ref } from "vue";

import { MasterDetail } from "../master-detail.ts";

type Mailbox = "inbox" | "sent" | "archive";

const mailboxes: readonly Mailbox[] = ["inbox", "sent", "archive"];
const selected = ref<Mailbox | null>(null);
</script>

<template>
  <MasterDetail
    v-model:selected="selected"
    :ssr-width="1024"
    master-label="Mailboxes"
    detail-label="Messages"
  >
    <template #master="{ select, selected: current }">
      <ul>
        <li v-for="mailbox in mailboxes" :key="mailbox">
          <button
            type="button"
            :aria-current="current === mailbox ? 'true' : undefined"
            @click="() => select(mailbox)"
          >
            {{ mailbox }}
          </button>
        </li>
      </ul>
    </template>
    <template #detail="{ selected: current, back }">
      <h2>{{ current }}</h2>
      <button type="button" @click="back">Back to mailboxes</button>
    </template>
    <template #empty>
      <p>Choose a mailbox to read its messages.</p>
    </template>
  </MasterDetail>
</template>
