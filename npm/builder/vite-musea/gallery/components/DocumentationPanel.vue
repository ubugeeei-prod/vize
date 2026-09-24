<script setup lang="ts">
import { ref, watch } from "vue";
import MarkdownContent from "./MarkdownContent";
import { fetchDocs } from "../api";

const props = defineProps<{ artPath: string }>();
const markdown = ref("");
const loading = ref(false);
const error = ref<string | null>(null);

watch(
  () => props.artPath,
  async (path, _previousPath, onCleanup) => {
    let cancelled = false;
    onCleanup(() => {
      cancelled = true;
    });
    markdown.value = "";
    error.value = null;
    if (!path) return;

    loading.value = true;
    try {
      const data = await fetchDocs(path);
      if (!cancelled) markdown.value = data.markdown;
    } catch (cause) {
      if (!cancelled) error.value = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (!cancelled) loading.value = false;
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="docs-panel">
    <div v-if="loading" class="docs-loading" role="status">
      <div class="loading-spinner" aria-hidden="true" />
      Loading documentation...
    </div>

    <div v-else-if="error" class="docs-error" role="alert">
      {{ error }}
    </div>

    <div v-else-if="markdown" class="docs-content">
      <MarkdownContent :markdown />
    </div>

    <div v-else class="docs-empty">
      <p>No documentation available for this component.</p>
    </div>
  </div>
</template>

<style scoped>
.docs-panel {
  padding: 0.5rem;
}

.docs-loading {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  justify-content: center;
  min-height: 200px;
  color: var(--musea-text-muted);
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--musea-border);
  border-block-start-color: var(--musea-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@media (prefers-reduced-motion: reduce) {
  .loading-spinner {
    animation-duration: 2s;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.docs-error {
  padding: 1rem;
  color: var(--musea-error);
  background: rgba(248, 113, 113, 0.1);
  border: 1px solid rgba(248, 113, 113, 0.2);
  border-radius: var(--musea-radius-md);
  font-size: 0.8125rem;
}

.docs-content {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
}

.docs-markdown {
  padding: 1.5rem;
  font-size: 0.875rem;
  line-height: 1.7;
  color: var(--musea-text-secondary);

  & :deep(h1) {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--musea-text);
    margin-block-end: 1rem;
    padding-bottom: 0.5rem;
    border-block-end: 1px solid var(--musea-border);
  }

  & :deep(h2) {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--musea-text);
    margin-block-start: 1.5rem;
    margin-block-end: 0.75rem;
  }

  & :deep(h3) {
    font-size: 1rem;
    font-weight: 600;
    color: var(--musea-text);
    margin-block-start: 1.25rem;
    margin-block-end: 0.5rem;
  }

  & :deep(p) {
    margin-block-end: 0.75rem;
  }

  & :deep(ul),
  & :deep(ol) {
    padding-inline-start: 1.5rem;
    margin-block-end: 0.75rem;
  }

  & :deep(li) {
    margin-block-end: 0.25rem;
  }

  & :deep(code) {
    background: var(--musea-bg-tertiary);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    font-family: "SF Mono", "Fira Code", "Consolas", monospace;
    font-size: 0.8125rem;
  }

  & :deep(pre) {
    background: var(--musea-bg-primary);
    border: 1px solid var(--musea-border);
    border-radius: var(--musea-radius-md);
    padding: 1rem;
    margin-block-end: 1rem;
    overflow-x: auto;
    white-space: pre;
  }

  & :deep(pre code) {
    background: none;
    padding: 0;
    font-size: 0.8125rem;
    line-height: 1.6;
    white-space: pre;
    tab-size: 2;
  }

  & :deep(table) {
    width: 100%;
    border-collapse: collapse;
    margin-block-end: 1rem;
  }

  & :deep(th),
  & :deep(td) {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--musea-border);
    text-align: left;
    font-size: 0.8125rem;
  }

  & :deep(th) {
    background: var(--musea-bg-tertiary);
    font-weight: 600;
    color: var(--musea-text);
  }

  & :deep(blockquote) {
    border-inline-start: 3px solid var(--musea-accent);
    padding-inline-start: 1rem;
    margin: 0.75rem 0;
    color: var(--musea-text-muted);
  }

  & :deep(hr) {
    border: none;
    border-block-start: 1px solid var(--musea-border);
    margin: 1.5rem 0;
  }

  & :deep(a) {
    color: var(--musea-accent);
    text-decoration: underline;
  }

  & :deep(strong) {
    color: var(--musea-text);
    font-weight: 600;
  }
}

.docs-empty {
  padding: 2rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
}
</style>
