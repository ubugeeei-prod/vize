<!-- Dual listbox that moves team members between "Available" and "Assigned" with search and bulk moves. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  TransferList,
  TransferListAction,
  TransferListEmpty,
  TransferListItem,
  TransferListPanel,
  TransferListSearch,
} from "../transfer-list.ts";

const people = ["Ada", "Alan", "Grace", "Katherine", "Linus", "Margaret"];
const assigned = ref<readonly string[]>(["Grace"]);
const availableId = useId();
const assignedId = useId();
</script>

<template>
  <TransferList
    v-slot="{ visibleSource, visibleTarget }"
    v-model="assigned"
    :items="people"
    name="assignees"
  >
    <div>
      <span :id="availableId">Available</span>
      <TransferListSearch side="source" aria-label="Search available people" />
      <TransferListPanel side="source" :aria-labelledby="availableId">
        <TransferListItem v-for="person in visibleSource" :key="person" :value="person">
          {{ person }}
        </TransferListItem>
      </TransferListPanel>
      <TransferListEmpty side="source">Nobody left to assign</TransferListEmpty>
    </div>
    <div>
      <TransferListAction action="move-selected-to-target">&gt;</TransferListAction>
      <TransferListAction action="move-all-to-target">&gt;&gt;</TransferListAction>
      <TransferListAction action="move-selected-to-source">&lt;</TransferListAction>
      <TransferListAction action="move-all-to-source">&lt;&lt;</TransferListAction>
    </div>
    <div>
      <span :id="assignedId">Assigned</span>
      <TransferListPanel side="target" :aria-labelledby="assignedId">
        <TransferListItem v-for="person in visibleTarget" :key="person" :value="person">
          {{ person }}
        </TransferListItem>
      </TransferListPanel>
      <TransferListEmpty side="target">Nobody assigned yet</TransferListEmpty>
    </div>
  </TransferList>
</template>
