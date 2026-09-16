<script setup lang="ts">
import { computed } from "vue";
import {
  featuresFor,
  type ExperimentalOptions,
  type ExperimentalScope,
  type ExperimentalKey,
} from "./experimentalFeatures";

const props = defineProps<{ scope: ExperimentalScope; modelValue: ExperimentalOptions }>();
const emit = defineEmits<{
  "update:modelValue": [options: ExperimentalOptions];
  example: [key: string];
}>();
const features = computed(() => featuresFor(props.scope));
const enabledCount = computed(
  () => features.value.filter(({ key }) => props.modelValue[key]).length,
);

function toggle(key: ExperimentalKey, event: Event) {
  emit("update:modelValue", {
    ...props.modelValue,
    [key]: (event.target as HTMLInputElement).checked,
  });
}
function selectExample(event: Event) {
  const select = event.target as HTMLSelectElement;
  emit("example", select.value);
  select.value = "";
}
</script>

<template>
  <details class="experimental-features">
    <summary>
      Experimental <span v-if="enabledCount">({{ enabledCount }})</span>
    </summary>
    <fieldset>
      <legend class="sr-only">Experimental features</legend>
      <div class="experimental-choices">
        <label v-for="feature in features" :key="feature.key">
          <input
            type="checkbox"
            :checked="modelValue[feature.key] === true"
            @change="toggle(feature.key, $event)"
          />
          <span>{{ feature.label }}</span>
        </label>
      </div>
      <select aria-label="Experimental example" @change="selectExample">
        <option value="">Example...</option>
        <option v-for="feature in features" :key="feature.key" :value="feature.key">
          {{ feature.label }}
        </option>
      </select>
    </fieldset>
  </details>
</template>

<style scoped>
.experimental-features {
  flex: 0 0 auto;
  border-bottom: 1px solid var(--border-color);
  padding: 8px 16px;
  font-size: 12px;
}
.experimental-features :deep(summary) {
  cursor: pointer;
  width: fit-content;
}
.experimental-features :deep(fieldset) {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px 16px;
  border: 0;
  margin: 10px 0 2px;
  padding: 0;
  min-width: 0;
}
.experimental-features :deep(.experimental-choices) {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
}
.experimental-features :deep(label) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 28px;
}
.experimental-features :deep(input) {
  accent-color: var(--accent-rust);
}
.experimental-features :deep(select) {
  max-width: 100%;
  min-height: 30px;
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 4px 8px;
  font: inherit;
}
.experimental-features :deep(.sr-only) {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
</style>
