<script setup lang="ts">
defineOptions({ name: "TypeCheckPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
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

/* Syntax highlighting colors */
.help-content {
  & .hl-keyword {
    color: var(--hl-keyword);
  }
}

.help-content {
  & .hl-vue-api {
    color: var(--hl-function);
  }
}

.help-content {
  & .hl-string {
    color: var(--hl-string);
  }
}

.help-content {
  & .hl-comment {
    color: var(--hl-comment);
    font-style: italic;
  }
}

.help-content {
  & .hl-tag {
    color: var(--hl-tag);
  }
}

.help-content {
  & .hl-directive {
    color: var(--hl-directive);
  }
}

.help-content {
  & .hl-delimiter {
    color: var(--hl-delimiter);
  }
}

.help-content {
  & .hl-type {
    color: var(--hl-type);
  }
}

.help-content {
  & .hl-number {
    color: var(--hl-number);
  }
}

.help-content {
  & .hl-property {
    color: var(--hl-property);
  }
}

.help-content {
  & .hl-value {
    color: var(--hl-value);
  }
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

.severity-info {
  & .severity-icon {
    color: var(--color-info);
  }
}

.diagnostic-code {
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

/* Virtual TS Output */
.virtualts-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.virtual-ts-notice {
  padding: 0.5rem 0.75rem;
  background: var(--color-notice-bg);
  border: 1px solid var(--color-notice-border);
  border-radius: 4px;
  font-size: 0.6875rem;
  color: var(--color-notice);
  line-height: 1.4;
  margin-bottom: 0.5rem;
}

.virtualts-output {
  & .editor-container {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
}

.virtualts-output {
  & .code-highlight {
    min-height: 100%;
  }
}

/* Capabilities Output */
.capabilities-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.capability-section {
  & h3 {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0 0 0.5rem 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
}

.capability-section {
  & p {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin: 0.25rem 0 0 0;
  }
}

.mode-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  font-size: 0.75rem;
  color: var(--color-info);
}

.checks-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.check-item {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.check-name {
  font-size: 0.75rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
}

.check-severity {
  margin-inline-start: 0.5rem;
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  text-transform: uppercase;
}

.check-severity.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.check-severity.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.check-description {
  font-size: 0.6875rem;
  color: var(--text-muted);
  margin: 0.25rem 0 0 0;
}

.notes-list {
  margin: 0;
  padding-inline-start: 1rem;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.notes-list {
  & li {
    margin: 0.25rem 0;
  }
}

.empty-state,
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
  .typecheck-playground {
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

  .options-panel {
    flex-direction: column;
  }
}
</style>
