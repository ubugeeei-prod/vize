<!-- Dismissible, titled maintenance banner controlled with v-model:open. -->
<script setup lang="ts">
import { ref } from "vue";

import { Banner } from "../banner.ts";

const open = ref(true);
</script>

<template>
  <div>
    <Banner
      v-model:open="open"
      title="Scheduled maintenance"
      description="Deployments are paused on Saturday from 02:00 to 04:00 UTC."
      tone="warning"
      dismissible
      dismiss-label="Dismiss maintenance notice"
    >
      <template #actions>
        <a href="#status">View status page</a>
      </template>
    </Banner>
    <button v-if="!open" type="button" @click="() => (open = true)">Show maintenance notice</button>
  </div>
</template>
