<script setup lang="ts">
defineOptions({ name: "CroquisPlaygroundStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.croquis-playground {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  height: 100%;
  min-height: 0;
  grid-column: 1 / -1;
  background: var(--bg-primary);
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.input-panel {
  border-right: 1px solid var(--border-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-title {
  & .icon {
    width: 1rem;
    height: 1rem;
    color: var(--accent-rust);
  }
}

.header-title {
  & h2 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0;
  }
}

.perf-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border-radius: 3px;
  font-family: "JetBrains Mono", monospace;
}

.panel-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-ghost {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-ghost:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  transition: all 0.15s;
}

.toggle-label:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.toggle-label {
  & input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: var(--accent-primary);
  }
}

.toggle-label {
  & span {
    white-space: nowrap;
  }
}

.tabs {
  display: flex;
  gap: 0.125rem;
}

.tab {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.tab.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
  font-weight: 500;
}

.tab-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.3125rem;
  background: var(--color-error-bg);
  color: var(--color-error);
  border-radius: 8px;
  min-width: 1rem;
  text-align: center;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

.output-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Error State */
.error-panel {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  border-radius: 6px;
  overflow: hidden;
  margin: 1rem;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: var(--color-error-bg);
  color: var(--color-error);
  font-size: 0.75rem;
  font-weight: 600;
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: var(--color-error);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

/* Empty/Loading/Success States */
.empty-state,
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  margin: 1rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--color-success);
  font-size: 0.875rem;
}

.success-icon {
  width: 1.25rem;
  height: 1.25rem;
}

/* Stats Tab */
.stats-output,
.bindings-output,
.scopes-output,
.diagnostics-output {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0.75rem;
  margin-bottom: 1.5rem;
}

.stat-box {
  padding: 1rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  text-align: center;
}

.stat-number {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--accent-rust);
  font-family: "JetBrains Mono", monospace;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
}

.section {
  margin-bottom: 1.5rem;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-primary);
}

.macro-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.macro-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.macro-name {
  font-weight: 600;
  font-family: "JetBrains Mono", monospace;
  color: var(--accent-rust);
}

.macro-type {
  font-size: 0.75rem;
  color: var(--color-info);
  background: var(--color-info-bg);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.macro-binding {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.css-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.css-stat {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.css-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
}

.css-badge.scoped {
  background: var(--color-purple-bg);
  color: var(--color-purple);
}

.css-badge.vbind {
  background: var(--color-teal-bg);
  color: var(--color-teal);
}

/* Export sections */
.export-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.export-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
}

.export-item.valid {
  background: var(--color-success-bg);
}

.export-item.invalid {
  background: var(--color-error-bg);
}

.export-kind {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  background: var(--surface-elevated);
  color: var(--text-muted);
}

.export-name {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.export-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
  margin-inline-start: auto;
}
</style>
