<script setup lang="ts">
defineOptions({ name: "InspectorPlaygroundStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.inspector-file-list {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  overflow-x: auto;
}

.inspector-file-tab {
  max-width: 220px;
  padding: 5px 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.inspector-file-tab:hover,
.inspector-file-tab.active {
  background: var(--bg-tertiary);
  border-color: var(--border-color);
  color: var(--text-primary);
}

.inspector-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.inspector-targets {
  display: flex;
  gap: 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 2px;
}

.inspector-target {
  border: 0;
  background: transparent;
  color: var(--text-muted);
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 0.75rem;
  cursor: pointer;
}

.inspector-target.active {
  background: var(--bg-panel);
  color: var(--text-primary);
}

.inspector-pr-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 30px;
  padding: 5px 12px;
  border: 1px solid color-mix(in srgb, var(--text-primary) 28%, transparent);
  border-radius: 6px;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background 0.15s ease;
}

.inspector-pr-link:hover {
  border-color: var(--text-primary);
  background: var(--bg-panel);
}

.inspector-pr-link {
  & svg {
    width: 15px;
    height: 15px;
    fill: currentColor;
    flex-shrink: 0;
  }
}

.inspector-option {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  cursor: pointer;
  user-select: none;
}

.inspector-option {
  & input {
    accent-color: var(--accent-rust);
  }
}

.inspector-option {
  & select {
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-panel);
    color: var(--text-primary);
    font: inherit;
    padding: 2px 6px;
  }
}

.inspector-tab-panel {
  min-height: 0;
}

.inspector-diff-toolbar {
  display: inline-flex;
  gap: 4px;
  margin-bottom: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-tertiary);
  padding: 2px;
}

.inspector-diff-mode {
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.75rem;
  padding: 4px 10px;
}

.inspector-diff-mode.active {
  background: var(--bg-panel);
  color: var(--text-primary);
}

.inspector-diff {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-secondary);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  line-height: 1.55;
}

.inspector-diff-line {
  display: grid;
  grid-template-columns: 46px 46px 22px minmax(0, 1fr);
  min-height: 22px;
  border-bottom: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-diff-line:last-child {
  border-bottom: 0;
}

.inspector-diff-line.add {
  background: var(--color-success-bg);
}

.inspector-diff-line.remove {
  background: var(--color-error-bg);
}

.inspector-diff-line.same {
  background: transparent;
}

.inspector-diff-num,
.inspector-diff-mark {
  padding: 2px 6px;
  color: var(--text-muted);
  text-align: right;
  user-select: none;
}

.inspector-diff-mark {
  text-align: center;
}

.inspector-diff-split {
  overflow-x: auto;
}

.inspector-split-line {
  display: grid;
  grid-template-columns: 46px minmax(280px, 1fr) 46px minmax(280px, 1fr);
  min-width: 720px;
  min-height: 22px;
  border-bottom: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-split-line:last-child {
  border-bottom: 0;
}

.inspector-split-code {
  border-left: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-split-code.remove {
  background: var(--color-error-bg);
}

.inspector-split-code.add {
  background: var(--color-success-bg);
}

.inspector-split-code.empty {
  background: color-mix(in srgb, var(--bg-tertiary) 45%, transparent);
}

.inspector-diff-code {
  padding: 2px 10px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
}

.inspector-diff-code {
  & span {
    color: var(--l, inherit);
  }
}

body[data-theme="dark"] {
  & .inspector-diff-code span {
    color: var(--d, inherit);
  }
}

.inspector-empty-diff {
  padding: 24px;
  color: var(--text-muted);
  text-align: center;
}

.inspector-warning-list {
  margin-bottom: 12px;
}

.inspector-warning {
  padding: 8px 10px;
  border: 1px solid color-mix(in srgb, var(--color-warning) 35%, transparent);
  border-radius: 6px;
  background: var(--color-warning-bg);
  color: var(--color-warning);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  white-space: pre-wrap;
}

.inspector-warning {
  & + .inspector-warning {
    margin-top: 6px;
  }
}

.inspector-payload {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 12px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  white-space: pre-wrap;
  word-break: break-word;
}

.inspector-vir-output {
  display: flex;
  min-height: 0;
  flex-direction: column;
}

.inspector-vir-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-bottom: 0;
  border-radius: 6px 6px 0 0;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
}

.inspector-vir-content {
  min-height: 280px;
  max-height: 58vh;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 0 0 6px 6px;
  background: var(--bg-secondary);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  line-height: 1.55;
}

.inspector-vir-line {
  display: flex;
  white-space: pre;
}

.inspector-vir-ln {
  min-width: 42px;
  padding: 0 8px;
  border-right: 1px solid var(--border-color);
  color: var(--text-muted);
  text-align: right;
  user-select: none;
}

.inspector-vir-line-text {
  padding: 0 10px;
}

.vir-token {
  color: var(--text-secondary);
}

.vir-section,
.vir-section-name,
.vir-binding,
.vir-identifier {
  color: var(--text-primary);
}

.vir-macro {
  color: var(--accent-rust);
}

.vir-type,
.vir-source,
.vir-keyword,
.vir-number,
.vir-colon,
.vir-bracket,
.vir-tag,
.vir-arrow,
.vir-border {
  color: var(--text-muted);
}

.vir-line-diagnostic {
  background: var(--color-error-bg);
}

.inspector-graph-panel {
  display: flex;
  min-height: 0;
  flex-direction: column;
  gap: 10px;
}

.inspector-graph-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
</style>
