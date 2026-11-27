<!-- Website field with a protocol addon and a copy button. -->
<script setup lang="ts">
import { ref } from "vue";

import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from "../index.ts";

const site = ref("vize.dev");
const copies = ref(0);
</script>

<template>
  <div>
    <InputGroup label="Website">
      <InputGroupAddon>https://</InputGroupAddon>
      <InputGroupInput v-model="site" type="url" />
      <InputGroupButton @press="copies += 1">Copy</InputGroupButton>
    </InputGroup>
    <output>Copied {{ copies }}</output>
  </div>
</template>
