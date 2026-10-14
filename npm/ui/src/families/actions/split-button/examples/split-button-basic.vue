<!-- Save action with related save options in an attached menu. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  SplitButton,
  SplitButtonAction,
  SplitButtonContent,
  SplitButtonItem,
  SplitButtonSeparator,
  SplitButtonTrigger,
} from "../split-button.ts";

const lastAction = ref("none");
</script>

<template>
  <div>
    <SplitButton aria-label="Save options">
      <SplitButtonAction @press="() => (lastAction = 'Save')">Save</SplitButtonAction>
      <SplitButtonTrigger aria-label="More save options">▾</SplitButtonTrigger>
      <SplitButtonContent>
        <SplitButtonItem @select="() => (lastAction = 'Save as draft')">
          Save as draft
        </SplitButtonItem>
        <SplitButtonItem @select="() => (lastAction = 'Save and publish')">
          Save and publish
        </SplitButtonItem>
        <SplitButtonSeparator />
        <SplitButtonItem @select="() => (lastAction = 'Schedule')">Schedule…</SplitButtonItem>
      </SplitButtonContent>
    </SplitButton>
    <output>Last action: {{ lastAction }}</output>
  </div>
</template>
