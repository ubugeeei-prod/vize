<!-- Presence host that mounts and unmounts details through explicit enter and exit phases. -->
<script setup lang="ts">
import { nextTick, ref, useId, useTemplateRef } from "vue";

import { Presence } from "../presence.ts";

const open = ref(false);
const detailsId = useId();
const presence = useTemplateRef<InstanceType<typeof Presence>>("presence");

async function toggle(): Promise<void> {
  open.value = !open.value;
  await nextTick();
  // No CSS animation is attached here, so finish the phase explicitly.
  presence.value?.completeAnimation();
}
</script>

<template>
  <div>
    <button type="button" :aria-expanded="open" :aria-controls="detailsId" @click="toggle">
      Shipping details
    </button>
    <Presence ref="presence" :present="open">
      <template #default="{ status }">
        <p :id="detailsId" :data-status="status">Orders ship within two business days.</p>
      </template>
    </Presence>
  </div>
</template>
