<script setup lang="ts">
defineOptions({ name: "CroquisPlaygroundStyles03" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
/* VIR Tab */
.vir-output {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 1rem;
}

.vir-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px 4px 0 0;
  border-bottom: none;
}

.vir-title {
  font-size: 0.6875rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.vir-line-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.vir-notice {
  padding: 0.5rem 0.75rem;
  background: var(--color-notice-bg);
  border: 1px solid var(--color-notice-border);
  font-size: 0.6875rem;
  color: var(--color-notice);
  line-height: 1.4;
}

.vir-content {
  flex: 1;
  min-height: 0;
  margin: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-bottom: none;
  font-size: 0.8125rem;
  font-family: "JetBrains Mono", monospace;
  overflow: auto;
  line-height: 1.6;
}

.vir-ln {
  padding: 0 0.75rem;
  text-align: right;
  color: var(--text-muted);
  font-size: 0.8125rem;
  min-width: 2.5rem;
  opacity: 0.6;
  height: 1.6em;
  line-height: 1.6;
  display: inline-block;
  background: rgba(0, 0, 0, 0.15);
  border-right: 1px solid var(--border-primary);
  user-select: none;
}

.vir-code {
  flex: 1;
  padding: 0.75rem 0;
}

.vir-line {
  display: flex;
  white-space: pre;
  height: 1.6em;
  line-height: 1.6;
}

.vir-line-text {
  padding: 0 1rem;
}

/* VIR Token Types - Subtle syntax highlighting */
.vir-token {
  color: var(--text-secondary);
}

.vir-border {
  color: var(--text-muted);
  opacity: 0.5;
}

.vir-section {
  color: var(--text-primary);
  font-weight: 600;
}

.vir-section-name {
  color: var(--text-primary);
  font-weight: 600;
}

.vir-macro {
  color: var(--color-purple);
}

.vir-type {
  color: var(--text-muted);
  font-style: italic;
}

.vir-binding {
  color: var(--text-primary);
}

.vir-identifier {
  color: var(--text-primary);
}

.vir-tag {
  color: var(--text-muted);
}

.vir-source {
  color: var(--text-muted);
  font-style: italic;
}

.vir-arrow {
  color: var(--text-muted);
}

.vir-number {
  color: var(--text-muted);
}

.vir-diagnostic {
  font-weight: 500;
}

.vir-line-diagnostic {
  & .vir-diagnostic {
    color: var(--color-error);
  }
}

.vir-keyword {
  color: var(--text-muted);
}

.vir-colon {
  color: var(--text-muted);
}

.vir-bracket {
  color: var(--text-muted);
}

.vir-plain {
  color: var(--text-secondary);
}

/* Line type background hints - removed for cleaner look */
.vir-line-section {
}

.vir-line-macro {
}

.vir-line-binding {
}

.vir-line-diagnostic {
  background: var(--color-error-bg);
}

/* Diagnostics Tab */
.diagnostic-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.diagnostic-item {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.diagnostic-header {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.severity-icon {
  width: 0.875rem;
  height: 0.875rem;
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

.diagnostic-message {
  flex: 1;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.diagnostic-location {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.375rem;
  padding-inline-start: 1.25rem;
}

.location-range {
  font-size: 0.75rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.diagnostic-code {
  font-size: 0.625rem;
  color: var(--text-secondary);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .croquis-playground {
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

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
