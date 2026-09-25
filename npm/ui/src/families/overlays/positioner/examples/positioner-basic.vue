<!-- Toggle a floating panel positioned below its trigger button with an arrow. -->
<script setup lang="ts">
import { ref, useId, useTemplateRef } from "vue";

import { Positioner, PositionerArrow } from "../positioner.ts";

const open = ref(false);
const panelId = useId();
const anchor = useTemplateRef<HTMLButtonElement>("anchor");

function toggle(): void {
  open.value = !open.value;
}
</script>

<template>
  <div>
    <button
      ref="anchor"
      type="button"
      :aria-expanded="open"
      :aria-controls="panelId"
      @click="toggle"
    >
      Account
    </button>
    <Positioner v-if="open" :reference="anchor" placement="bottom-start" :offset="8">
      <PositionerArrow />
      <div :id="panelId" role="group" aria-label="Account shortcuts">
        <a href="#profile">Profile</a>
        <a href="#billing">Billing</a>
      </div>
    </Positioner>
  </div>
</template>
