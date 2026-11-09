<script setup lang="ts">
defineOptions({ name: "GlyphPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.option-card:hover {
  border-color: var(--accent-rust);
}

.option-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
}

.option-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
}

.option-input {
  width: 60px;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: "JetBrains Mono", monospace;
}

.option-input:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.option-select {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: "JetBrains Mono", monospace;
  cursor: pointer;
}

.option-select:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.option-desc {
  font-size: 0.6875rem;
  color: var(--text-muted);
}

.toggle-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 0.5rem;
}

.toggle-grid--before-options {
  margin-block-end: 0.75rem;
}

.toggle-grid--after-options {
  margin-block-start: 0.75rem;
}

.toggle-card {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.625rem 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.toggle-card:hover {
  border-color: var(--accent-rust);
}

.toggle-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toggle-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-rust);
  cursor: pointer;
}

.toggle-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
}

.toggle-desc {
  font-size: 0.6875rem;
  color: var(--text-muted);
  padding-inline-start: 1.5rem;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .glyph-playground {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(300px, 1fr) minmax(300px, 1fr);
    height: auto;
    min-height: 100%;
  }

  .panel {
    min-height: 300px;
  }

  .input-panel {
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
  }

  .panel-header {
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tabs {
    flex-wrap: wrap;
    width: 100%;
  }

  .options-grid,
  .toggle-grid {
    grid-template-columns: 1fr;
  }
}
</style>
