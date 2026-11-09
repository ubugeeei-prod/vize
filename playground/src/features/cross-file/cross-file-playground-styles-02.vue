<script setup lang="ts">
defineOptions({ name: "CrossFilePlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.option-toggle {
  & input {
    width: 12px;
    height: 12px;
    accent-color: var(--accent-primary);
  }
}

.option-toggle:hover {
  color: var(--text-primary);
}

/* === Editor Pane === */
.editor-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  padding-inline-end: 12px;
}

.editor-tabs {
  display: flex;
  overflow-x: auto;
}

.editor-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: none;
  border-right: 1px solid var(--border-primary);
  font-size: 0.6875rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.1s;
}

.editor-tab:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.editor-tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-rust);
  margin-bottom: -1px;
}

.tab-icon {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
  flex-shrink: 0;
}

.tab-badge {
  font-size: 0.5rem;
  padding: 0 3px;
  border-radius: 4px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 12px;
}

.tab-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.tab-badge.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.editor-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
}

.status-chip {
  height: 18px;
  padding: 0 6px;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  display: inline-flex;
  align-items: center;
  color: var(--text-muted);
  background: var(--bg-primary);
}

.status-analyzing {
  color: var(--accent-rust);
}

.status-time {
  color: var(--text-muted);
}

.editor-content {
  flex: 1;
  min-height: 0;
  position: relative;
}

/* === Diagnostics Pane === */
.diagnostics-pane {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-primary);
  overflow: hidden;
}

.diagnostics-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-primary);
}

.diagnostics-header {
  & h3 {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin: 0;
  }
}

.diagnostics-stats {
  display: flex;
  gap: 6px;
  align-items: center;
}

.stat-chip {
  font-size: 0.5625rem;
  padding: 0 3px;
  border-radius: 3px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 14px;
}

.stat-chip.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.stat-chip.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.stat-chip.info {
  background: var(--color-info-bg);
  color: var(--color-info);
}

.filter-chip {
  height: 16px;
  padding: 0 6px;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 0.5625rem;
  font-weight: 600;
}

.filter-chip.active {
  border-color: var(--accent-rust);
  color: var(--accent-rust);
}

.severity-filters {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 4px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-primary);
}

.filter-btn {
  min-width: 0;
  height: 22px;
  padding: 0 4px;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 0.625rem;
  font-weight: 600;
}

.filter-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.filter-btn.active.error {
  border-color: var(--color-error);
  color: var(--color-error);
  background: var(--color-error-bg);
}

.filter-btn.active.warning {
  border-color: var(--color-warning);
  color: var(--color-warning);
  background: var(--color-warning-bg);
}

.filter-btn.active.info {
  border-color: var(--color-info);
  color: var(--color-info);
  background: var(--color-info-bg);
}

.diagnostics-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px;
  color: var(--color-success);
}

.empty-icon {
  width: 24px;
  height: 24px;
}

.diagnostics-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
}

.issue-group {
  margin-bottom: 8px;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  margin-bottom: 4px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.group-badge {
  font-size: 0.625rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.group-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.group-issues {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issue-card {
  padding: 5px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.1s;
}

.issue-card:hover {
  background: var(--bg-tertiary);
}

.issue-card.selected {
  border-color: var(--accent-rust);
  background: rgba(224, 112, 72, 0.1);
}

.issue-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.severity-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.issue-card.error {
  & .severity-icon {
    color: var(--color-error);
  }
}
.issue-card.warning {
  & .severity-icon {
    color: var(--color-warning);
  }
}
.issue-card.info {
  & .severity-icon {
    color: var(--color-info);
  }
}

.issue-code {
  font-size: 0.5625rem;
  font-family: "JetBrains Mono", monospace;
  padding: 1px 4px;
  background: var(--bg-secondary);
  border-radius: 2px;
  color: var(--text-muted);
}

.issue-location {
  margin-inline-start: auto;
  font-size: 0.5625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.issue-message {
  font-size: 0.6875rem;
  color: var(--text-primary);
  line-height: 1.4;
}

.issue-suggestion {
  margin-top: 4px;
  padding: 4px 6px;
  font-size: 0.625rem;
  color: var(--color-success);
  background: var(--color-success-bg);
  border-radius: 3px;
  display: flex;
  gap: 6px;
}

.suggestion-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}
</style>
