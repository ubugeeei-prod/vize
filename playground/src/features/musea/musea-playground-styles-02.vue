<script setup lang="ts">
defineOptions({ name: "MuseaPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.color-swatch {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.token-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.token-name {
  font-size: 0.6875rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.token-value {
  font-size: 0.5625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.token-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.size-token,
.other-token {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.625rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.size-token:hover,
.other-token:hover {
  border-color: var(--accent-rust);
}

:is(.size-token, .other-token) {
  & .token-name {
    flex: 1;
    font-size: 0.75rem;
  }
}

:is(.size-token, .other-token) {
  & .token-value {
    font-size: 0.6875rem;
    color: var(--accent-rust);
    min-width: 50px;
    text-align: right;
  }
}

.size-preview {
  height: 6px;
  background: var(--accent-rust);
  border-radius: 3px;
  min-width: 4px;
  max-width: 150px;
}

/* Variants Output */
.variants-output {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.variant-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.variants-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.variant-item {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  overflow: hidden;
}

.variant-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-primary);
}

.variant-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.default-badge {
  font-size: 0.5625rem;
  padding: 0.0625rem 0.375rem;
  background: rgba(163, 72, 40, 0.2);
  color: var(--accent-rust);
  border-radius: 2px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.skip-badge {
  font-size: 0.5625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--color-warning-bg);
  color: var(--color-warning);
  border: 1px solid var(--color-warning-border);
  border-radius: 2px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.btn-copy {
  padding: 0.125rem 0.375rem;
  font-size: 0.625rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-copy:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.variant-template {
  padding: 0.5rem;
}

/* CSF Output */
.csf-output {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.csf-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.filename-badge {
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.code-container {
  flex: 1;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  overflow: auto;
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
  .musea-playground {
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

  .metadata-grid {
    grid-template-columns: 1fr;
  }

  .color-grid {
    grid-template-columns: 1fr;
  }
}
</style>
