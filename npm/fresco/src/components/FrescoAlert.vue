<script setup lang="ts">
import { computed } from "vue";

import { alertConfigs, type AlertProps } from "./presentational-primitive-types.js";

const {
  message,
  type = "info",
  title,
  showIcon = true,
  border = "rounded",
} = defineProps<AlertProps>();

defineOptions({ name: "FrescoAlert" });

const config = computed(() => alertConfigs[type]);
const heading = computed(() => title ?? config.value.title);
const iconLabel = computed(() => `${config.value.icon} `);
const containerStyle = { flexDirection: "column", padding: 1 } as const;
const headerStyle = { flexDirection: "row" } as const;
const messageStyle = { marginTop: 1 } as const;
</script>

<template>
  <div :border="border === 'none' ? undefined : border" :fg="config.fg" :style="containerStyle">
    <div :style="headerStyle">
      <text v-if="showIcon" :fg="config.fg">{{ iconLabel }}</text>
      <text :fg="config.fg" :bold="true">{{ heading }}</text>
    </div>
    <text :style="messageStyle">{{ message }}</text>
  </div>
</template>
