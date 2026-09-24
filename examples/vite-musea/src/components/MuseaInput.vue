<script setup lang="ts">
import "../theme.css";
defineProps<{
  modelValue?: string;
  placeholder?: string;
  type?: "text" | "email" | "password" | "search";
  disabled?: boolean;
  error?: string;
  label: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

defineArt("./MuseaInput.vue", {
  title: "Input",
  category: "Forms",
  tags: ["input", "form", "text"],
  status: "ready",
});

function onInput(event: Event) {
  emit("update:modelValue", (event.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="input-wrapper">
    <input
      class="input"
      :class="{ 'input--error': error, 'input--disabled': disabled }"
      :type="type ?? 'text'"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :aria-label="label"
      @input="onInput"
    />
    <span v-if="error" class="input-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.input {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--musea-line);
  border-radius: 6px;
  font-size: 0.875rem;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  outline: none;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
  width: 100%;
  background: var(--musea-paper);
  color: var(--musea-ink);
}

.input::placeholder {
  color: var(--musea-muted);
}

.input:focus {
  border-color: var(--musea-ink);
  box-shadow: 0 0 0 3px var(--musea-ink-ring);
}

.input--error {
  border-color: var(--musea-error);
}

.input--error:focus {
  box-shadow: 0 0 0 3px var(--musea-error-ring);
}

.input--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--musea-surface);
}

.input-error {
  color: var(--musea-error);
  font-size: 0.75rem;
}
</style>

<art>
  <variant name="Default" default>
    <Self label="Text input" placeholder="Enter text..." />
  </variant>
  <variant name="With Value">
    <Self label="Text input" model-value="Hello, Musea!" placeholder="Enter text..." />
  </variant>
  <variant name="Search">
    <Self label="Search" type="search" placeholder="Search..." />
  </variant>
  <variant name="With Error">
    <Self label="Email" model-value="bad@" error="Invalid email address" placeholder="Enter email..." />
  </variant>
  <variant name="Disabled">
    <Self label="Disabled input" model-value="Read only" disabled placeholder="Disabled input" />
  </variant>
</art>
