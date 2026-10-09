<script setup lang="ts">
import { useGlobals } from "../composables/useGlobals";
import type { ResolvedMuseaToolbarControl } from "../../src/toolbar.js";

const { toolbar, globals, setGlobal } = useGlobals();

function select(control: ResolvedMuseaToolbarControl, event: Event) {
  const index = Number((event.target as HTMLSelectElement).value);
  const option = control.options[index];
  if (option) setGlobal(control.id, option.value);
}
</script>

<template>
  <div v-for="control in toolbar" :key="control.id" class="global-control">
    <label v-if="control.type === 'select'" class="global-select">
      <span>{{ control.title }}</span>
      <select
        :aria-label="control.title"
        :value="control.options.findIndex((option) => option.value === globals[control.id])"
        @change="(event) => select(control, event)"
      >
        <option
          v-for="(option, index) in control.options"
          :key="`${typeof option.value}:${option.value}`"
          :value="index"
        >
          {{ option.icon ? `${option.icon} ${option.label}` : option.label }}
        </option>
      </select>
    </label>
    <div v-else role="group" :aria-label="control.title" class="global-toggle">
      <span>{{ control.title }}</span>
      <button
        v-for="option in control.options"
        :key="`${typeof option.value}:${option.value}`"
        type="button"
        :aria-pressed="globals[control.id] === option.value"
        @click="() => setGlobal(control.id, option.value)"
      >
        <span v-if="option.icon" aria-hidden="true">{{ option.icon }}</span>
        {{ option.label }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.global-control,
.global-select,
.global-toggle {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--musea-text-secondary);
  font-size: 0.6875rem;
}
select,
button {
  font: inherit;
  border: 1px solid var(--musea-border);
  border-radius: 2px;
  background: var(--musea-bg-tertiary);
  color: var(--musea-text);
  padding: 0.25rem 0.375rem;
}
button {
  cursor: pointer;
}
button[aria-pressed="true"] {
  border-color: var(--musea-accent);
  background: var(--musea-accent-subtle);
  color: var(--musea-accent);
}
select:focus-visible,
button:focus-visible {
  outline: 2px solid var(--musea-accent);
  outline-offset: 2px;
}
</style>
