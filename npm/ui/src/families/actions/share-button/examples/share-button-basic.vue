<!-- Share button that hands a title, text, and URL to the Web Share API. -->
<script setup lang="ts">
import { ref } from "vue";

import { ShareButton } from "../share-button.ts";

const status = ref("");

function onShare(): void {
  status.value = "Shared";
}

function onError(): void {
  status.value = "Sharing is unavailable in this browser";
}
</script>

<template>
  <div>
    <ShareButton
      title="Vize documentation"
      text="Read the Vize docs"
      url="https://vizejs.dev"
      idle-label="Share docs"
      @share="onShare"
      @error="onError"
    />
    <output>{{ status }}</output>
  </div>
</template>
