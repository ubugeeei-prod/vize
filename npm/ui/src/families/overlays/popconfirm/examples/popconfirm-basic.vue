<!-- Delete button guarded by an anchored confirmation that runs an async handler before closing. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  Popconfirm,
  PopconfirmCancel,
  PopconfirmConfirm,
  PopconfirmContent,
  PopconfirmTrigger,
} from "../popconfirm.ts";

const deleted = ref(false);

async function deleteDraft(): Promise<void> {
  await Promise.resolve();
  deleted.value = true;
}
</script>

<template>
  <div>
    <Popconfirm :on-confirm="deleteDraft">
      <PopconfirmTrigger :disabled="deleted">Delete draft</PopconfirmTrigger>
      <PopconfirmContent
        title="Delete this draft?"
        description="The draft and its attachments are removed permanently."
      >
        <PopconfirmCancel>Keep draft</PopconfirmCancel>
        <PopconfirmConfirm>Delete</PopconfirmConfirm>
      </PopconfirmContent>
    </Popconfirm>
    <output>{{ deleted ? "Draft deleted" : "Draft saved" }}</output>
  </div>
</template>
