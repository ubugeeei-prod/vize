<script setup lang="ts">
import { computed } from "vue";
import { useA11y } from "../composables/useA11y";

const props = defineProps<{
  artPath: string;
  variantName?: string;
}>();

const { init, getResult, isKeyRunning } = useA11y();

init();

const key = computed(() => (props.variantName ? `${props.artPath}:${props.variantName}` : null));

const result = computed(() => (key.value ? getResult(key.value) : undefined));
const running = computed(() => (key.value ? isKeyRunning(key.value) : false));

const count = computed(() => {
  if (running.value) {
    return "…";
  }

  if (!result.value) {
    return null;
  }

  return String(result.value.violations.length);
});

const severity = computed(() => {
  if (running.value) {
    return "running";
  }

  if (!result.value) {
    return "none";
  }

  if (result.value.error) {
    return "critical";
  }

  if (result.value.violations.length === 0) {
    return "passed";
  }

  const hasCritical = result.value.violations.some((violation) => violation.impact === "critical");
  const hasSerious = result.value.violations.some((violation) => violation.impact === "serious");

  return hasCritical ? "critical" : hasSerious ? "serious" : "moderate";
});

const title = computed(() => {
  if (running.value) {
    return "Accessibility test is running";
  }

  if (!result.value) {
    return "";
  }

  if (result.value.error) {
    return result.value.error;
  }

  return `${result.value.violations.length} accessibility violation${
    result.value.violations.length !== 1 ? "s" : ""
  }`;
});
</script>

<template>
  <span v-if="count !== null" class="a11y-badge" :class="'severity-' + severity" :title="title">
    {{ count }}
  </span>
</template>

<style scoped>
.a11y-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 0.25rem;
  border-radius: 9px;
  font-size: 0.625rem;
  font-weight: 700;
  line-height: 1;
}

.severity-running {
  background: rgba(251, 191, 36, 0.16);
  color: var(--musea-warning);
}

.severity-passed {
  background: rgba(74, 222, 128, 0.18);
  color: #4ade80;
}

.severity-moderate {
  background: rgba(251, 191, 36, 0.2);
  color: var(--musea-warning);
}

.severity-serious {
  background: rgba(248, 113, 113, 0.2);
  color: var(--musea-error);
}

.severity-critical {
  background: rgba(248, 113, 113, 0.3);
  color: var(--musea-error);
}
</style>
