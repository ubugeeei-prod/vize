<script setup lang="ts">
import { computed } from "vue";

import type { CardProps } from "./presentational-primitive-types.js";

const {
  title,
  subtitle,
  footer,
  border = "rounded",
  padding = 1,
  titleFg = "white",
  borderFg,
  bg,
} = defineProps<CardProps>();

defineOptions({ name: "FrescoCard" });

const slots = defineSlots<{
  default?(): unknown;
  footer?(): unknown;
}>();
const hasHeader = computed(() => Boolean(title || subtitle));
const hasFooter = computed(() => Boolean(footer || slots.footer));
const containerStyle = computed(() => ({ flexDirection: "column" as const, padding }));
const headerStyle = { flexDirection: "row", marginBottom: 1 } as const;
const contentStyle = { flexGrow: 1 } as const;
const footerStyle = { marginTop: 1 } as const;
const subtitleLabel = computed(() => (title ? ` - ${subtitle}` : subtitle));
</script>

<template>
  <div :border="border === 'none' ? undefined : border" :fg="borderFg" :bg :style="containerStyle">
    <div v-if="hasHeader" :style="headerStyle">
      <text v-if="title" :fg="titleFg" :bold="true">{{ title }}</text>
      <text v-if="subtitle" :dim="true">{{ subtitleLabel }}</text>
    </div>
    <div :style="contentStyle"><slot /></div>
    <div v-if="hasFooter" :style="footerStyle">
      <slot name="footer"
        ><text :dim="true">{{ footer }}</text></slot
      >
    </div>
  </div>
</template>
