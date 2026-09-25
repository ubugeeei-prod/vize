<!-- Disclosure controlled with v-model:open that reveals advanced settings. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "../collapsible.ts";

const open = ref(false);
const inputId = useId();
</script>

<template>
  <CollapsibleRoot v-model:open="open">
    <CollapsibleTrigger>{{ open ? "Hide" : "Show" }} advanced settings</CollapsibleTrigger>
    <CollapsibleContent>
      <label :for="inputId">Request timeout (seconds)</label>
      <input :id="inputId" name="timeout" type="number" value="30" />
    </CollapsibleContent>
  </CollapsibleRoot>
</template>
