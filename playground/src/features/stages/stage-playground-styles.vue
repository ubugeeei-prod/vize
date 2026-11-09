<script setup lang="ts">
defineOptions({ name: "StagePlaygroundStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
/* Levels tab: source on the left, the stage ladder on the right. The one
   decorative device is the pounce line between stations - the pricked,
   dusted line a pounced transfer leaves - and everything else stays quiet. */

.level-playground {
  --dv-ochre: var(--color-notice);
  --dv-ochre-bg: var(--color-notice-bg);
  --dv-ochre-border: var(--color-notice-border);
  --dv-static: var(--color-teal);
  --dv-static-bg: var(--color-teal-bg);

  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: minmax(360px, 5fr) 7fr;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.level-source,
.level-stages {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.level-source {
  border-right: 1px solid var(--border-primary);
}

.level-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 48px;
  padding: 0 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.level-title {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.level-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.level-keys {
  margin-inline-start: auto;
  white-space: nowrap;
}

.level-source {
  & .level-hint {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.level-example {
  display: flex;
  align-items: center;
  gap: 6px;
}

.level-select {
  padding: 4px 8px;
  font: inherit;
  font-size: 0.75rem;
  color: var(--text-primary);
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
}

.level-ghost {
  padding: 4px 10px;
  font-size: 0.75rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
}

.level-ghost:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.level-badge {
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.level-editor {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Provenance highlight painted into Monaco (global: Monaco owns its DOM). */
.level-provenance {
  background: var(--color-notice-bg);
  outline: 1.5px solid var(--color-notice);
  outline-offset: -1px;
  border-radius: 2px;
}

/* Page tabs within a stage, and output targets */
.level-subtabs {
  display: flex;
  gap: 4px;
  padding: 10px 20px 0;
  flex-shrink: 0;
  overflow-x: auto;
}

/* A tab label never wraps: the row scrolls instead. */
.level-subtab {
  flex-shrink: 0;
  white-space: nowrap;
  padding: 6px 12px;
  font-size: 0.8125rem;
  font-family: var(--font-mono);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.level-subtab:hover {
  color: var(--text-secondary);
  background: var(--bg-secondary);
}

.level-subtab.active {
  color: var(--text-primary);
  border-color: var(--border-primary);
  background: var(--bg-secondary);
}

.level-subtabs-gap {
  flex: 1;
}

.level-count {
  margin-inline-start: 4px;
  padding: 0 6px;
  font-size: 0.6875rem;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.level-body {
  flex: 1;
  min-height: 0;
  padding: 12px 20px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.level-output {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  flex: 1;
}

.level-output {
  & .level-subtabs {
    padding: 0;
  }
}

.level-output {
  & .level-output-code.code-highlight {
    flex: 1;
    min-height: 0;
    --code-line-height: 22px;
    font-size: 0.875rem;
  }
}

.level-message {
  margin: 20px;
  padding: 14px 16px;
  font-size: 0.875rem;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
}

.level-message.error {
  color: var(--color-error);
  background: var(--color-error-bg);
  border-color: var(--color-error-border);
  white-space: pre-wrap;
}

.level-provenance-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 40px;
  padding: 0 20px;
  border-top: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  flex-shrink: 0;
  overflow: hidden;
}

.level-span {
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--dv-ochre);
  background: var(--dv-ochre-bg);
  border: 1px solid var(--dv-ochre-border);
  border-radius: 4px;
  white-space: nowrap;
}

/* Why the focused op exists: its provenance rules (lowering, then pass facts). */
.level-why {
  flex-shrink: 0;
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--hl-keyword);
  background: var(--bg-tertiary);
  border-radius: 4px;
  white-space: nowrap;
}

.level-why.fact {
  flex-shrink: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.level-why.remark.applied {
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.level-why.remark.missed {
  color: var(--color-notice);
  background: var(--color-notice-bg);
}

.level-snippet {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 960px) {
  .level-playground {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(240px, 40%) 1fr;
  }

  .level-source {
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
  }

  .level-timeline-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
