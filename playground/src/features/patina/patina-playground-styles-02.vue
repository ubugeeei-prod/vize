<script setup lang="ts">
defineOptions({ name: "PatinaPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
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

/* Rules Output */
.rules-output {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.rules-toolbar {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.rules-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.preset-actions {
  display: flex;
  gap: 0.375rem;
  margin-inline-end: 0.5rem;
}

.btn-action {
  padding: 0.25rem 0.5rem;
  font-size: 0.625rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-action:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border-color: var(--accent-rust);
}

.btn-preset.active {
  background: color-mix(in srgb, var(--accent-rust) 12%, var(--bg-tertiary));
  color: var(--text-primary);
  border-color: var(--accent-rust);
}

.search-input {
  flex: 1;
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.category-select {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
  cursor: pointer;
}

.category-select:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.category-toggle {
  padding: 0.625rem 0.75rem;
  margin-bottom: 0.75rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border: 1px solid var(--border-primary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.category-label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.category-count {
  font-size: 0.6875rem;
  color: var(--text-muted);
}

/* Rules List */
.rules-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.rule-item {
  padding: 0.625rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  transition: all 0.15s;
}

.rule-item:hover {
  border-color: var(--border-secondary);
}

.rule-item.disabled {
  opacity: 0.5;
}

.rule-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.rule-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.rule-checkbox {
  width: 14px;
  height: 14px;
  accent-color: var(--accent-rust);
  cursor: pointer;
}

.rules-output {
  & .rule-id {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-primary);
    background: transparent;
    padding: 0;
  }
}

.rule-item.disabled {
  & .rule-id {
    text-decoration: line-through;
  }
}

.rule-badges {
  display: flex;
  gap: 0.375rem;
  flex-wrap: wrap;
}

.badge {
  display: inline-block;
  padding: 0.0625rem 0.375rem;
  font-size: 0.5625rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: 2px;
}

.category-badge {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.severity-badge {
  background: var(--bg-tertiary);
}

.severity-badge.error {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-weight: 600;
}

.severity-badge.warning {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.fixable-badge {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.rule-description {
  font-size: 0.6875rem;
  color: var(--text-muted);
  line-height: 1.4;
  padding-inline-start: 1.375rem;
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
  .patina-playground {
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

  .rules-toolbar {
    flex-direction: column;
  }

  .preset-actions {
    width: 100%;
    margin-inline-end: 0;
  }

  .rule-main {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.375rem;
  }

  .rule-description {
    padding-inline-start: 0;
  }
}
</style>
