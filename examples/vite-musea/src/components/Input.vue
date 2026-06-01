<script setup lang="ts">
defineProps<{
  modelValue?: string;
  placeholder?: string;
  type?: "text" | "email" | "password" | "search";
  disabled?: boolean;
  error?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

defineArt("./Input.vue", {
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
  border: 1px solid #c8c4b8;
  border-radius: 6px;
  font-size: 0.875rem;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  outline: none;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
  width: 100%;
  background: #e6e2d6;
  color: #121212;
}

.input::placeholder {
  color: #9a9890;
}

.input:focus {
  border-color: #121212;
  box-shadow: 0 0 0 3px rgba(18, 18, 18, 0.08);
}

.input--error {
  border-color: #a04040;
}

.input--error:focus {
  box-shadow: 0 0 0 3px rgba(160, 64, 64, 0.12);
}

.input--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: #ddd9cd;
}

.input-error {
  color: #a04040;
  font-size: 0.75rem;
}
</style>

<art>
  <variant name="Default" default>
    <Self placeholder="Enter text..." />
  </variant>
  <variant name="With Value">
    <Self model-value="Hello, Musea!" placeholder="Enter text..." />
  </variant>
  <variant name="Search">
    <Self type="search" placeholder="Search..." />
  </variant>
  <variant name="With Error">
    <Self model-value="bad@" error="Invalid email address" placeholder="Enter email..." />
  </variant>
  <variant name="Disabled">
    <Self model-value="Read only" disabled placeholder="Disabled input" />
  </variant>
</art>
