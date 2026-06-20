<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  value: string | number;
  tokenType: "fontSize" | "fontWeight" | "lineHeight";
}>();

const style = computed(() => {
  const val = String(props.value);
  switch (props.tokenType) {
    case "fontSize":
      return {
        fontSize:
          val.includes("px") || val.includes("rem") || val.includes("em") ? val : val + "px",
      };
    case "fontWeight":
      return { fontWeight: val };
    case "lineHeight":
      return { lineHeight: val };
    default:
      return {};
  }
});

const isLineHeight = computed(() => props.tokenType === "lineHeight");
</script>

<template>
  <div class="typography-preview">
    <template v-if="isLineHeight">
      <div class="typography-sample" :style="style">Aa<br />Bb</div>
    </template>
    <template v-else>
      <div class="typography-sample" :style="style">Aa</div>
    </template>
  </div>
</template>

<style scoped>
.typography-preview {
  display: flex;
  align-items: center;
  width: 48px;
  height: 48px;
  overflow: hidden;
}

.typography-sample {
  color: var(--musea-text);
  font-family: var(--musea-font-mono);
  line-height: 1.2;
  white-space: nowrap;
}
</style>
