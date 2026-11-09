<script setup lang="ts">
defineOptions({ name: "PatinaPlaygroundStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.patina-playground {
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

.count-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  border-radius: 8px;
  min-width: 1.25rem;
  text-align: center;
  font-family: "JetBrains Mono", monospace;
}

.count-badge.errors {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-weight: 600;
}

.count-badge.warnings {
  background: var(--bg-tertiary);
  color: var(--text-muted);
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
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-radius: 8px;
  min-width: 1rem;
  text-align: center;
  font-weight: 600;
}

.tab-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

.output-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

/* Error State */
.error-panel {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  overflow: hidden;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 600;
  border-bottom: 1px solid var(--border-primary);
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: "JetBrains Mono", monospace;
}

/* Output Header Bar */
.output-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-primary);
  margin-bottom: 0.75rem;
}

.output-title {
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.locale-selector {
  & select {
    padding: 0.25rem 0.5rem;
    font-size: 0.625rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: 3px;
    color: var(--text-primary);
    cursor: pointer;
  }
}

/* Success State */
.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.success-icon {
  width: 1.25rem;
  height: 1.25rem;
}

/* Diagnostics List */
.diagnostics-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.diagnostic-item {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-left: 2px solid var(--text-muted);
  border-radius: 0;
}

.diagnostic-item.severity-error {
  border-left-color: var(--color-error);
}

.diagnostic-item.severity-warning {
  border-left-color: var(--color-warning);
}

.diagnostic-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.375rem;
}

.severity-icon {
  width: 0.75rem;
  height: 0.75rem;
  flex-shrink: 0;
}

.severity-error {
  & .severity-icon {
    color: var(--color-error);
  }
}

.severity-warning {
  & .severity-icon {
    color: var(--color-warning);
  }
}

.rule-id {
  font-size: 0.6875rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.location-badge {
  margin-inline-start: auto;
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.diagnostic-message {
  font-size: 0.8125rem;
  color: var(--text-primary);
  line-height: 1.4;
}

.diagnostic-help {
  margin-top: 0.75rem;
  padding: 0.75rem;
  background: var(--bg-tertiary);
  border-left: 2px solid var(--text-muted);
  border-radius: 0;
  font-size: 0.85rem;
}

.help-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-primary);
}

.help-icon {
  font-size: 1rem;
}

.help-label {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.8125rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.help-content {
  color: var(--text-primary);
  line-height: 1.6;
}

.help-content {
  & strong {
    color: var(--text-primary);
    font-weight: 600;
  }
}

.help-content {
  & .help-code {
    margin: 0.5rem 0;
    padding: 1rem 1.125rem;
    background: var(--code-bg);
    border: 1px solid var(--code-border);
    border-radius: 6px;
    overflow-x: auto;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.8125rem;
    line-height: 1.55;
  }
}

.help-content {
  & .help-code code {
    color: var(--code-foreground);
    background: none;
    padding: 0;
  }
}

.help-content {
  & .help-inline-code {
    background: var(--code-bg);
    border: 1px solid var(--code-border);
    padding: 0.18rem 0.42rem;
    border-radius: 3px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.85em;
    color: var(--hl-keyword);
  }
}

/* Syntax highlighting colors — warm earthy tones, theme-adaptive */
.help-content {
  & .hl-keyword {
    color: var(--hl-keyword);
  }
}
</style>
