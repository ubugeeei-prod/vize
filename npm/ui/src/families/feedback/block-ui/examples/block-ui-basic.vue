<!-- Settings region made inert and busy while a save is in progress, with a polite status label. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { BlockUI } from "../block-ui.ts";

const saving = ref(false);
const inputId = useId();
</script>

<template>
  <div>
    <BlockUI
      v-slot="{ blocked }"
      as="div"
      :blocked="saving"
      reason="saving"
      interaction="inert"
      announce="polite"
      :label="saving ? 'Saving settings' : ''"
    >
      <label :for="inputId">Display name</label>
      <input :id="inputId" name="display-name" value="Ada Lovelace" />
      <p v-if="blocked">Saving your changes…</p>
    </BlockUI>
    <button type="button" @click="() => (saving = !saving)">
      {{ saving ? "Finish saving" : "Save settings" }}
    </button>
  </div>
</template>
