<!-- Controlled switch bound with v-model and labelled by visible text. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Switch } from "../switch.ts";

const enabled = ref(false);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Email notifications</span>
    <Switch v-model="enabled" name="notifications" :aria-labelledby="labelId" />
    <output>{{ enabled ? "on" : "off" }}</output>
  </div>
</template>
