<script setup lang="ts">
defineOptions({ name: "AppStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.panel-header {
  & h2 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 10px;
  }
}

.compile-time {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: var(--font-mono);
}

.panel-actions {
  display: flex;
  gap: 8px;
}

.btn-ghost {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  padding: 4px 12px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-ghost:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.tabs {
  display: flex;
  gap: 4px;
  width: 100%;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.tabs::-webkit-scrollbar {
  display: none;
}

.tab-copy-btn {
  margin-inline-start: auto;
}

.tab {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.8125rem;
  padding: 6px 14px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: all 0.2s;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.tab.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

/* Editor */
/* Note: .editor-container is defined in .panel section above with min-height: 0 */

/* Output */
.output-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.compiling {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  color: var(--text-muted);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-rust);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.wasm-error {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  border-radius: var(--radius-lg);
  padding: 16px;
}

.wasm-error {
  & h3 {
    color: var(--color-error);
    font-size: 0.875rem;
    margin-bottom: 12px;
  }
}

.wasm-error {
  & pre {
    color: var(--color-error);
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    white-space: pre-wrap;
    word-break: break-word;
  }
}

.wasm-warning {
  background: var(--color-warning-bg);
  border: 1px solid color-mix(in srgb, var(--color-warning) 35%, transparent);
  border-radius: var(--radius-lg);
  padding: 16px;
  margin-bottom: 16px;
}

.wasm-warning {
  & h3 {
    color: var(--color-warning);
    font-size: 0.875rem;
    margin-bottom: 12px;
  }
}

.wasm-warning {
  & pre {
    color: var(--color-warning);
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    white-space: pre-wrap;
    word-break: break-word;
  }
}

.wasm-warning {
  & pre + pre {
    margin-top: 8px;
  }
}

:is(.code-output, .ast-output, .helpers-output) {
  & h4 {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    margin-bottom: 0.75rem;
  }
}

/* AST Options */
.ast-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.ast-header {
  & h4 {
    margin-bottom: 0;
  }
}

.ast-options {
  display: flex;
  align-items: center;
  gap: 16px;
}

.ast-option {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.75rem;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
}

.ast-option {
  & input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: var(--accent-rust);
    cursor: pointer;
  }
}

.ast-option:hover {
  color: var(--text-primary);
}

.btn-small {
  padding: 2px 8px;
  font-size: 0.6875rem;
}

.preamble {
  margin-bottom: 24px;
}

/* Generated code surfaces override syntax highlighter inline presentation. */
/* vize-disable css/no-important */
:is(.code-output, .ast-output) {
  & pre {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    line-height: 1.45;
    background: var(--code-bg) !important;
    border: 1px solid var(--code-border);
    border-radius: var(--radius-lg);
    color: var(--code-foreground);
    padding: 20px !important;
    overflow: auto;
    margin: 0;
  }
}
/* vize-enable css/no-important */

.line-number {
  display: inline-block;
  width: 30px;
  color: var(--text-muted);
  user-select: none;
  text-align: right;
  margin-inline-end: 16px;
  font-size: 0.75rem;
}

.helpers-list {
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.helper-item {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  padding: 0.25rem 0.5rem;
  transition: all 0.15s;
}

.helper-item:hover {
  border-color: var(--border-primary);
}

.helper-name {
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--text-secondary);
}

/* Footer */
.footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px 24px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border-color);
  font-size: 0.75rem;
  color: var(--text-muted);
}

.footer {
  & a {
    color: var(--accent-metal);
    text-decoration: none;
    transition: color 0.2s;
  }
}

.footer {
  & a:hover {
    color: var(--accent-rust);
  }
}

.divider,
.separator {
  opacity: 0.3;
}

.rust-badge {
  & .rust {
    color: var(--text-primary);
    font-weight: 600;
  }
}

.oxc-badge {
  & .oxc {
    color: var(--text-primary);
    font-weight: 600;
  }
}

.code-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 8px;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.code-header {
  & h4 {
    margin-bottom: 0;
  }
}

.code-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.code-mode-toggle {
  display: flex;
  gap: 4px;
  background: var(--bg-tertiary);
  border-radius: var(--radius);
  padding: 2px;
}

.code-view-toggle {
  min-width: 90px;
}

.toggle-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: calc(var(--radius) - 2px);
  cursor: pointer;
  transition: all 0.2s;
}
</style>
