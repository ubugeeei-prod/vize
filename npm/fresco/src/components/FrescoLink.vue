<script setup lang="ts">
import type { LinkProps } from "./display-primitive-types.js";

const { text, url, fg = "blue", underline = true, showUrl = false } = defineProps<LinkProps>();
defineEmits<{ click: [] }>();

defineOptions({ name: "FrescoLink" });

const rowStyle = { flexDirection: "row" } as const;
</script>

<template>
  <div :style="rowStyle">
    <text :fg :underline>{{ text }}</text>
    <text v-if="showUrl && url" :dim="true"> ({{ url }})</text>
  </div>
</template>
