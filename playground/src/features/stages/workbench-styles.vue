<script setup lang="ts">
defineOptions({ name: "DavinciWorkbenchStyles" });
</script>

<!-- Shared selectors style the IR panes and Monaco's generated decorations. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.dv-workbench {
  --dv-layer-traces: 3;
  --dv-layer-resizer: 5;
  --dv-l1: var(--color-teal);
  --dv-l2: var(--hl-keyword);
  --dv-l3: var(--color-notice);
  --dv-l4: var(--hl-function);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-secondary);
}
.dv-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 9px 16px 7px;
  flex-shrink: 0;
}
.dv-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  h2 {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 650;
    letter-spacing: -0.6px;
    span {
      margin-inline-start: 8px;
      color: var(--text-muted);
      font-weight: 400;
    }
  }
}
.dv-controls {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}
.dv-toggle {
  display: flex;
  gap: 2px;
  padding: 3px;
  border: 1px solid var(--border-primary);
  border-radius: 7px;
  background: var(--bg-primary);
  button {
    padding: 5px 9px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    font: 11px var(--font-mono);
    cursor: pointer;
    white-space: nowrap;
  }
  button.active {
    color: var(--text-primary);
    background: var(--bg-tertiary);
    box-shadow: 0 1px 3px #0001;
  }
  button:hover {
    color: var(--text-primary);
  }
}
.dv-workbench {
  button:focus-visible {
    outline: 2px solid var(--dv-l1);
    outline-offset: 2px;
  }
  select:focus-visible {
    outline: 2px solid var(--dv-l1);
    outline-offset: 2px;
  }
}
.dv-run-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px 5px;
  font: 10px var(--font-mono);
  color: var(--text-muted);
  b {
    margin: 0 6px;
    font-weight: 400;
    opacity: 0.5;
  }
  .davinci-badge {
    padding: 0;
    background: none;
    color: var(--text-secondary);
    font-size: inherit;
  }
}
.dv-live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--dv-l1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-run-hint {
  margin-inline-start: auto;
}
.dv-workspace {
  position: relative;
  display: grid;
  flex: 1;
  min-height: 0;
  margin: 0 14px 12px;
  border: 1px solid var(--border-primary);
  border-radius: 9px;
  overflow: auto;
  background: var(--bg-primary);
  [data-pane] {
    position: relative;
  }
  .davinci-stages {
    overflow: visible;
  }
}
.dv-resizer {
  position: absolute;
  top: 0;
  bottom: 0;
  inset-inline-start: -5px;
  width: 10px;
  z-index: var(--dv-layer-resizer);
  cursor: col-resize;
  touch-action: none;
  outline: none;
}
.dv-resizer::after {
  content: "";
  position: absolute;
  inset-inline-start: 4px;
  top: calc(50% - 16px);
  width: 2px;
  height: 32px;
  border-radius: 2px;
  background: var(--text-muted);
  opacity: 0.5;
}
.dv-resizer:hover {
  background: color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-resizer:focus-visible {
  background: color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-resizer.dragging {
  background: color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-resizer:hover::after {
  background: var(--dv-l1);
  opacity: 1;
}
.dv-resizer:focus-visible::after {
  background: var(--dv-l1);
  opacity: 1;
}
.dv-resizer.dragging::after {
  background: var(--dv-l1);
  opacity: 1;
}
.dv-workspace:has(.dv-resizer.dragging) {
  cursor: col-resize;
  user-select: none;
  .davinci-folio {
    pointer-events: none;
  }
  .davinci-editor {
    pointer-events: none;
  }
}
.dv-workbench {
  .davinci-source {
    background: var(--bg-primary);
    .davinci-bar {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      min-height: 76px;
      padding: 12px;
      align-content: center;
      background: var(--bg-primary);
    }
    .davinci-title {
      font-size: 0.875rem;
    }
  }
}
.dv-source-tag {
  font: 9px var(--font-mono);
  letter-spacing: 0.5px;
  padding: 4px 5px;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-muted);
}
.dv-workbench {
  .davinci-example {
    flex: 1;
    min-width: 100px;
    select {
      max-width: 100%;
      width: 100%;
      font-size: 0.625rem;
    }
  }
  .davinci-source {
    .davinci-ghost {
      padding: 3px 6px;
      font-size: 0.625rem;
    }
  }
}
.dv-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
}
.davinci-layers {
  display: contents;
}
.dv-trace-inspector {
  flex-shrink: 0;
  min-height: 0;
  max-height: 115px;
  overflow: auto;
  padding: 0 16px 8px;
}
.dv-trace-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 23px;
}
.dv-trace-label {
  font: 9px var(--font-mono);
  letter-spacing: 1.3px;
  color: var(--text-muted);
}
.dv-trace-heading {
  .davinci-hint {
    margin-inline-start: auto;
    font: 9px var(--font-mono);
  }
  .davinci-span {
    padding: 2px 5px;
    font-size: 0.5625rem;
  }
  .davinci-ghost {
    flex-shrink: 0;
    padding: 2px 5px;
    font-size: 0.5625rem;
  }
}
.dv-trace-content {
  display: flex;
  flex: 1;
  min-width: 0;
  align-items: center;
  overflow-x: auto;
  white-space: nowrap;
  gap: 8px;
}
.dv-trace-source {
  flex-shrink: 0;
  color: var(--text-primary);
  font: 11px var(--font-mono);
  overflow-wrap: anywhere;
}
.dv-trace-content {
  .davinci-why {
    font-size: 0.625rem;
    padding: 3px 6px;
  }
}
.dv-rule-arrow {
  opacity: 0.5;
  padding: 0 3px;
}
.dv-trace-note {
  margin-block-start: 8px;
  font-size: 0.625rem;
  color: var(--color-notice);
}
.dv-workbench {
  .davinci-provenance {
    background: color-mix(in srgb, var(--dv-l1) 15%, transparent);
    outline-color: var(--dv-l1);
  }
}
.dv-static-dot {
  color: var(--dv-l1);
}
.dv-dynamic-dot {
  color: var(--dv-l3);
}
.dv-dump-notice {
  margin: 4px 0 0;
  font-size: 0.5625rem;
  color: var(--text-muted);
  line-height: 1.5;
}
.dv-trace-links {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: var(--dv-layer-traces);
  path {
    fill: none;
    stroke: currentColor;
    stroke-width: 3;
    stroke-linecap: round;
  }
  .dv-trace-outline {
    stroke: var(--bg-primary);
    stroke-width: 7;
  }
  circle {
    fill: currentColor;
    stroke: var(--bg-primary);
    stroke-width: 2;
  }
}
.dv-link-l1 {
  color: var(--dv-l1);
}
.dv-link-l2 {
  color: var(--dv-l2);
}
.dv-link-l3 {
  color: var(--dv-l3);
}
@media (max-width: 1050px) {
  .dv-toolbar {
    flex-wrap: wrap;
    padding: 12px 14px;
  }
  .dv-brand {
    h2 {
      font-size: 1.0625rem;
    }
  }
  .dv-run-hint {
    display: none;
  }
  .dv-trace-heading {
    flex-wrap: wrap;
  }
}
</style>
