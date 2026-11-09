<script setup lang="ts">
defineOptions({ name: "CrossFilePlaygroundStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.cross-file-playground {
  display: grid;
  grid-template-columns: 220px 4px 1fr 4px 320px;
  grid-column: 1 / -1;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
  font-size: 0.75rem;
  user-select: none;
}

.cross-file-playground.resizing {
  cursor: col-resize;
}

.cross-file-playground.resizing {
  & * {
    pointer-events: none;
  }
}

/* === Resize Handles === */
.resize-handle {
  width: 4px;
  background: var(--border-primary);
  cursor: col-resize;
  transition: background 0.15s;
  position: relative;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--accent-primary);
}

.resize-handle::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 8px;
  inset-inline-start: -2px;
}

/* === Sidebar === */
.sidebar {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-primary);
  overflow: hidden;
}

.sidebar-section {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.sidebar-section:not(:last-child) {
  border-bottom: 1px solid var(--border-primary);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--bg-tertiary);
}

.section-header {
  & h3 {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin: 0;
  }
}

.analysis-mode-badge {
  font-size: 0.5625rem;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--color-success);
  color: var(--bg-primary);
  letter-spacing: 0.5px;
  cursor: help;
}

.profile-switch {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 4px;
  padding: 8px 12px 0;
}

.profile-btn {
  min-width: 0;
  height: 24px;
  padding: 0 4px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 0.625rem;
  font-weight: 600;
}

.profile-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.profile-btn.active {
  border-color: var(--accent-rust);
  color: var(--accent-rust);
  background: rgba(224, 112, 72, 0.12);
}

/* Preset Selector */
.preset-section {
  flex-shrink: 0;
}

.preset-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px;
}

.preset-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.625rem;
  color: var(--text-secondary);
  transition: all 0.15s;
  flex: 1;
  min-width: calc(50% - 4px);
  justify-content: flex-start;
}

.preset-item:hover {
  background: var(--bg-tertiary);
  border-color: var(--text-muted);
  color: var(--text-primary);
}

.preset-item.active {
  background: rgba(224, 112, 72, 0.15);
  border-color: var(--accent-rust);
  color: var(--accent-rust);
}

.preset-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.preset-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.section-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  & svg {
    width: 14px;
    height: 14px;
  }
}

.icon-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.75rem;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

/* File Tree */
.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.1s;
  position: relative;
}

.file-item:hover {
  background: var(--bg-tertiary);
}

.file-item.active {
  background: var(--accent-primary);
  background: rgba(224, 112, 72, 0.15);
}

.file-item.has-errors {
  & .file-icon {
    color: var(--color-error);
  }
}
.file-item.has-warnings {
  & .file-icon {
    color: var(--color-warning);
  }
}

.file-icon {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
  flex-shrink: 0;
}

.file-name {
  flex: 1;
  font-family: "JetBrains Mono", monospace;
  font-size: 0.6875rem;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-badge {
  font-size: 0.5rem;
  padding: 0 3px;
  border-radius: 4px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 12px;
}

.file-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.file-badge.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.file-delete {
  position: absolute;
  inset-inline-end: 8px;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  visibility: hidden;
  opacity: 0;
  pointer-events: none;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 2px;
}

.file-delete {
  & svg {
    width: 12px;
    height: 12px;
  }
}

.file-item:hover {
  & .file-delete {
    visibility: visible;
    opacity: 1;
    pointer-events: auto;
  }
}

.file-delete:hover {
  background: var(--color-error-bg);
  color: var(--color-error);
}

/* Dependency Graph */
.dependency-graph {
  padding: 8px 12px;
  font-family: "JetBrains Mono", monospace;
  font-size: 0.625rem;
}

.dep-node {
  margin-bottom: 8px;
}

.dep-file {
  color: var(--text-secondary);
}

.dep-arrows {
  padding-inline-start: 12px;
  margin-top: 2px;
}

.dep-edge {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
}

.dep-arrow {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
}

.dep-target {
  color: var(--text-secondary);
  cursor: pointer;
}

.dep-target:hover {
  color: var(--accent-rust);
  text-decoration: underline;
}

/* Options */
.options-section {
  margin-top: auto;
}

.options-grid {
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 0.6875rem;
  color: var(--text-secondary);
}
</style>
