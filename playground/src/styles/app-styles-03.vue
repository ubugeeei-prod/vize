<script setup lang="ts">
defineOptions({ name: "AppStyles03" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.toggle-btn:hover {
  color: var(--text-secondary);
}

.toggle-btn.active {
  background: var(--text-primary);
  color: var(--bg-primary);
}

.toggle-btn:disabled {
  cursor: default;
  color: var(--text-muted);
  opacity: 0.45;
}

.toggle-btn:disabled:hover {
  color: var(--text-muted);
}

.render-function {
  position: relative;
}

.no-helpers {
  color: var(--text-muted);
  font-style: italic;
}

/* SFC Output */
:is(.sfc-output, .css-output) {
  & h4 {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 16px;
  }
}

/* Generated SFC output overrides syntax highlighter inline presentation. */
/* vize-disable css/no-important */
.sfc-block {
  margin-bottom: 24px;
}

:is(.sfc-block, .css-compiled, .css-vars, .css-errors) {
  & h5 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }
}

.section-label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.sfc-block {
  & pre {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    line-height: 1.45;
    background: var(--code-bg) !important;
    border: 1px solid var(--code-border);
    border-radius: var(--radius-lg);
    color: var(--code-foreground);
    padding: 18px !important;
    overflow: auto;
    margin: 0;
  }
}
/* vize-enable css/no-important */

.style-block {
  margin-bottom: 12px;
}

.style-meta {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.badge {
  font-size: 0.6875rem;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(184, 189, 201, 0.15);
  color: var(--accent-metal);
}

/* CSS Output Tab */
.css-output {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.css-options {
  display: flex;
  gap: 20px;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-radius: var(--radius);
  border: 1px solid var(--border-color);
}

/* Generated CSS output overrides syntax highlighter inline presentation. */
/* vize-disable css/no-important */
.css-compiled {
  position: relative;
}

.css-compiled {
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

.css-vars {
  margin-top: 8px;
}

.css-errors {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  border-radius: var(--radius);
  padding: 12px;
}

.css-errors {
  & h5 {
    color: var(--color-error);
  }
}

.css-errors {
  & .error-message {
    color: var(--color-error);
    font-family: var(--font-mono);
    font-size: 0.75rem;
    margin: 8px 0 0 0;
    white-space: pre-wrap;
  }
}

.no-css {
  color: var(--text-muted);
  font-style: italic;
  text-align: center;
  padding: 40px 20px;
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

/* Monaco owns these nodes and applies equivalent styles inline. */
/* vize-disable css/no-important */
.monaco-editor {
  & .margin {
    background: transparent !important;
  }
}

.monaco-editor {
  padding-top: 0 !important;
}
/* vize-enable css/no-important */

/* Monaco Container - moved from scoped styles due to Vize compiler issue */
.monaco-container {
  width: 100%;
  height: 100%;
  min-height: 0;
  position: absolute;
  top: 0;
  inset-inline-start: 0;
  inset-inline-end: 0;
  bottom: 0;
}

/* Responsive */
@media (max-width: 1024px) {
  .main {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }

  .input-panel {
    border-right: none;
    border-bottom: 1px solid var(--border-color);
  }

  .options {
    flex-wrap: wrap;
    gap: 12px;
  }
}

@media (max-width: 640px) {
  .header {
    flex-direction: column;
    gap: 12px;
    padding: 12px 16px;
  }

  .logo {
    gap: 8px;
  }

  .logo-icon {
    width: 32px;
    height: 32px;
  }

  .logo-text {
    & h1 {
      font-size: 1.125rem;
    }
  }

  .options {
    width: 100%;
    justify-content: center;
    gap: 8px;
  }

  .option {
    font-size: 0.75rem;
  }

  .option {
    & select {
      padding: 4px 8px;
      font-size: 0.75rem;
    }
  }

  .panel-header {
    flex-wrap: wrap;
    gap: 8px;
    padding: 10px 12px;
  }

  .panel-header {
    & h2 {
      font-size: 0.8125rem;
      width: 100%;
    }
  }

  .panel-actions {
    order: 3;
  }

  .tab {
    font-size: 0.75rem;
    padding: 5px 10px;
  }

  .btn-ghost {
    font-size: 0.6875rem;
    padding: 4px 8px;
  }

  .output-content {
    padding: 12px;
  }

  .code-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .footer {
    padding: 10px 16px;
    font-size: 0.6875rem;
  }
}

/* Bindings Tab - Professional UI */
.bindings-output {
  & h4 {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    margin-bottom: 0.75rem;
  }
}

.bindings-summary {
  display: flex;
  gap: 0.375rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.summary-card {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.625rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 3px;
}

.summary-count {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.summary-type {
  font-size: 0.625rem;
  color: var(--text-muted);
}

.bindings-groups {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.binding-group {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-primary);
}

.group-icon {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  font-size: 0.625rem;
  font-weight: 600;
  font-family: var(--font-mono);
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
}
</style>
