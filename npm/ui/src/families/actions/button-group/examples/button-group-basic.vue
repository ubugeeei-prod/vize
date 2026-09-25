<!-- Text-formatting toolbar with arrow-key roving focus and one disabled action. -->
<script setup lang="ts">
import { ref } from "vue";

import { ButtonGroup, ButtonGroupItem } from "../button-group.ts";

const lastAction = ref("none");
</script>

<template>
  <div>
    <ButtonGroup
      role="toolbar"
      aria-label="Text formatting"
      @press="(value) => (lastAction = value)"
    >
      <ButtonGroupItem value="bold">Bold</ButtonGroupItem>
      <ButtonGroupItem value="italic">Italic</ButtonGroupItem>
      <ButtonGroupItem value="underline">Underline</ButtonGroupItem>
      <ButtonGroupItem value="strikethrough" disabled>Strikethrough</ButtonGroupItem>
    </ButtonGroup>
    <output>Last action: {{ lastAction }}</output>
  </div>
</template>
