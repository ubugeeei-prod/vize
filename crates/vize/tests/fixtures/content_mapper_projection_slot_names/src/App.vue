<script setup lang="ts">
import { ref } from "vue";
import Panel from "./Panel.vue";

const open = ref<boolean | undefined>(true);
const rows = ref([1, 2]);
</script>

<template>
  <Panel>
    <template #title="{ text }">{{ text }}</template>
    <template #subtitle>no such slot</template>
    <template #item-row="{ index }">{{ index }}</template>
    <template #item-col>no such quoted slot</template>
  </Panel>
  <Panel v-if="open">
    <template #footer>no such slot under v-if</template>
  </Panel>
  <Panel v-else>
    <template #title>{{ open }}</template>
    <template #aside>no such slot under v-else</template>
  </Panel>
  <Panel v-for="row in rows" :key="row">
    <template #title>
      <Panel>
        <template #nested>{{ row }}</template>
      </Panel>
    </template>
    <template #side>{{ row.missing }}</template>
  </Panel>
</template>
