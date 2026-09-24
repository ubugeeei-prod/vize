<!-- Controlled text input bound with v-model and labelled by a native label. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Input } from "../input.ts";

const name = ref("");
const inputId = useId();
</script>

<template>
  <div>
    <label :for="inputId">Display name</label>
    <Input
      :id="inputId"
      v-model="name"
      name="displayName"
      autocomplete="nickname"
      placeholder="Ada Lovelace"
    />
    <output :for="inputId">Hello, {{ name || "stranger" }}</output>
  </div>
</template>
