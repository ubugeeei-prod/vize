<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const model = defineModel<string>({ default: "" });
const inputRef = ref<HTMLInputElement | null>(null);

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    inputRef.value?.focus();
  }
  if (e.key === "Escape" && document.activeElement === inputRef.value) {
    model.value = "";
    inputRef.value?.blur();
  }
}

onMounted(() => {
  document.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="search-container">
    <svg
      class="search-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.35-4.35" />
    </svg>
    <input
      ref="inputRef"
      v-model="model"
      type="text"
      class="search-input"
      placeholder="Search components... (⌘K)"
    />
    <kbd v-if="!model" class="search-kbd">⌘K</kbd>
  </div>
</template>

<style scoped>
.search-container {
  position: relative;
  width: 280px;
}

.search-input {
  width: 100%;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  padding: 0.5rem 0.75rem 0.5rem 2.25rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  outline: none;
  transition:
    border-color var(--musea-transition),
    background var(--musea-transition);
}

.search-input::placeholder {
  color: var(--musea-text-muted);
}

.search-input:focus {
  border-color: var(--musea-accent);
  background: var(--musea-bg-elevated);
}

.search-icon {
  position: absolute;
  left: 0.75rem;
  top: 50%;
  transform: translateY(-50%);
  color: var(--musea-text-muted);
  pointer-events: none;
}

.search-kbd {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: var(--musea-bg-elevated);
  border: 1px solid var(--musea-border);
  border-radius: 4px;
  padding: 0.0625rem 0.375rem;
  font-size: 0.625rem;
  color: var(--musea-text-muted);
  font-family: inherit;
  pointer-events: none;
}
</style>
