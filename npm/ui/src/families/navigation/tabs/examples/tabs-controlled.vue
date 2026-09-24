<!-- Controlled vertical tabs bound with v-model using manual activation. -->
<script setup lang="ts">
import { ref } from "vue";

import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "../tabs.ts";
import type { TabsValue } from "../tabs.ts";

const section = ref<TabsValue>("general");
</script>

<template>
  <div>
    <TabsRoot v-model="section" orientation="vertical" activation-mode="manual">
      <TabsList aria-label="Settings">
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="privacy">Privacy</TabsTrigger>
      </TabsList>
      <TabsContent value="general">Language and time zone preferences.</TabsContent>
      <TabsContent value="privacy">Control who can see your activity.</TabsContent>
    </TabsRoot>
    <output>Current section: {{ section }}</output>
  </div>
</template>
