<script setup lang="ts">
defineOptions({ name: "DavinciWorkbenchStyles" });
</script>

<!-- Shared selectors style the IR panes and Monaco's generated decorations. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.dv-workbench {
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
}
.dv-brand h2 {
  margin: 0;
  font-size: 17px;
  font-weight: 650;
  letter-spacing: -0.6px;
}
.dv-brand h2 span {
  margin-left: 8px;
  color: var(--text-muted);
  font-weight: 400;
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
}
.dv-toggle button {
  padding: 5px 9px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  font: 11px var(--font-mono);
  cursor: pointer;
  white-space: nowrap;
}
.dv-toggle button.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
  box-shadow: 0 1px 3px #0001;
}
.dv-toggle button:hover {
  color: var(--text-primary);
}
.dv-workbench button:focus-visible,
.dv-workbench select:focus-visible {
  outline: 2px solid var(--dv-l1);
  outline-offset: 2px;
}
.dv-run-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px 5px;
  font: 10px var(--font-mono);
  color: var(--text-muted);
}
.dv-run-strip b {
  margin: 0 6px;
  font-weight: 400;
  opacity: 0.5;
}
.dv-run-strip .davinci-badge {
  padding: 0;
  background: none;
  color: var(--text-secondary);
  font-size: inherit;
}
.dv-live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--dv-l1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-run-hint {
  margin-left: auto;
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
}
.dv-workspace [data-pane] {
  position: relative;
}
.dv-workspace .davinci-stages {
  overflow: visible;
}
.dv-resizer {
  position: absolute;
  top: 0;
  bottom: 0;
  left: -5px;
  width: 10px;
  z-index: 5;
  cursor: col-resize;
  touch-action: none;
  outline: none;
}
.dv-resizer::after {
  content: "";
  position: absolute;
  left: 4px;
  top: calc(50% - 16px);
  width: 2px;
  height: 32px;
  border-radius: 2px;
  background: var(--text-muted);
  opacity: 0.5;
}
.dv-resizer:hover,
.dv-resizer:focus-visible,
.dv-resizer.dragging {
  background: color-mix(in srgb, var(--dv-l1) 12%, transparent);
}
.dv-resizer:hover::after,
.dv-resizer:focus-visible::after,
.dv-resizer.dragging::after {
  background: var(--dv-l1);
  opacity: 1;
}
.dv-workspace:has(.dv-resizer.dragging) {
  cursor: col-resize;
  user-select: none;
}
.dv-workspace:has(.dv-resizer.dragging) .davinci-folio,
.dv-workspace:has(.dv-resizer.dragging) .davinci-editor {
  pointer-events: none;
}
.dv-workbench .davinci-source {
  background: var(--bg-primary);
}
.dv-workbench .davinci-source .davinci-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 76px;
  padding: 12px;
  align-content: center;
  background: var(--bg-primary);
}
.dv-workbench .davinci-source .davinci-title {
  font-size: 14px;
}
.dv-source-tag {
  font: 9px var(--font-mono);
  letter-spacing: 0.5px;
  padding: 4px 5px;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-muted);
}
.dv-workbench .davinci-example {
  flex: 1;
  min-width: 100px;
}
.dv-workbench .davinci-example select {
  max-width: 100%;
  width: 100%;
  font-size: 10px;
}
.dv-workbench .davinci-source .davinci-ghost {
  padding: 3px 6px;
  font-size: 10px;
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
.dv-layer {
  --layer-color: var(--dv-l1);
  display: flex;
  flex-direction: column;
  position: relative;
  min-width: 0;
  min-height: 0;
  border-right: 1px solid var(--border-primary);
}
.dv-l2 {
  --layer-color: var(--dv-l2);
}
.dv-l3 {
  --layer-color: var(--dv-l3);
}
.dv-l4 {
  --layer-color: var(--dv-l4);
  border-right: 0;
}
.dv-layer-header {
  position: relative;
  flex-shrink: 0;
  padding: 12px 11px 10px;
  min-height: 76px;
  border-bottom: 1px solid var(--border-primary);
  background: var(--bg-primary);
}
.dv-layer-heading {
  display: flex;
  align-items: center;
  gap: 7px;
}
.dv-layer-heading h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  letter-spacing: -0.3px;
}
.dv-layer-alias {
  font: 10px var(--font-mono);
  color: var(--text-muted);
}
.dv-layer-header p {
  margin: 7px 0 8px;
  color: var(--text-muted);
  font-size: 10px;
}
.dv-layer-facts {
  display: flex;
  justify-content: space-between;
  gap: 4px;
  font: 9px var(--font-mono);
  color: var(--text-muted);
}
.dv-layer-facts code {
  color: var(--layer-color);
}
.dv-expand {
  margin-left: auto;
  border: none;
  background: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 2px;
}
.dv-expand:hover {
  color: var(--layer-color);
}
.dv-page-picker {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 9px;
  min-height: 38px;
  border-bottom: 1px solid var(--border-primary);
}
.dv-page-picker select {
  min-width: 0;
  flex: 1;
  padding: 3px 5px;
  border: 0;
  background: var(--bg-secondary);
  font: 10px var(--font-mono);
}
.dv-match-count {
  flex-shrink: 0;
  font: 9px var(--font-mono);
  color: var(--text-muted);
}
.dv-match-count.matched {
  color: var(--layer-color);
}
.dv-layer .davinci-folio {
  border: 0;
  border-radius: 0;
  padding: 10px 0;
  font-size: 10.5px;
  line-height: 1.9;
}
.dv-layer .davinci-line {
  padding-right: 8px;
}
.dv-layer .davinci-ln {
  width: 2.7em;
  padding-right: 8px;
  font-size: 9px;
}
.dv-layer .davinci-mark {
  width: 4px;
  height: 4px;
  margin-left: 4px;
}
.dv-layer .davinci-code {
  min-width: 0;
}
.dv-wrap .dv-layer .davinci-line {
  min-width: 0;
}
.dv-wrap .dv-layer .davinci-code {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.dv-layer .davinci-line.linked,
.dv-layer .davinci-line.selected {
  background: color-mix(in srgb, var(--layer-color) 10%, var(--code-bg));
  border-left-color: var(--layer-color);
}
.dv-layer .davinci-line.linkable:hover {
  background: color-mix(in srgb, var(--layer-color) 7%, var(--code-bg));
}
.dv-workbench .page-surface .davinci-line.linked {
  background: transparent;
  border-left-color: transparent;
}
.dv-surface-unit {
  display: inline;
  border: 0;
  border-radius: 2px;
  padding: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  white-space: inherit;
  text-align: inherit;
  cursor: pointer;
}
.dv-surface-unit.linked,
.dv-surface-unit:hover,
.dv-surface-unit:focus-visible {
  background: color-mix(in srgb, var(--dv-l1) 20%, var(--code-bg));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--dv-l1) 65%, transparent);
}
.dv-layer .davinci-output {
  gap: 0;
  overflow: hidden;
}
.dv-layer .davinci-output .davinci-subtabs {
  gap: 2px;
  min-height: 38px;
  padding: 5px;
  border-bottom: 1px solid var(--border-primary);
}
.dv-layer .davinci-output .davinci-subtab {
  font-size: 9px;
  padding: 4px 5px;
}
.dv-layer .davinci-output .davinci-output-code.code-highlight {
  font-size: 10.5px;
  --code-line-height: 20px;
  border: 0;
  border-radius: 0;
}
.dv-layer-footer {
  display: flex;
  justify-content: space-between;
  gap: 5px;
  padding: 8px 9px;
  border-top: 1px solid var(--border-primary);
  font: 9px var(--font-mono);
  color: var(--text-muted);
  background: var(--bg-secondary);
}
.dv-layer-empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 12px;
  text-align: center;
  padding: 20px;
  background-image: radial-gradient(var(--border-primary) 0.7px, transparent 0.7px);
  background-size: 12px 12px;
  color: var(--text-muted);
  font-size: 11px;
}
.dv-layer-empty p {
  margin: 0;
  line-height: 1.7;
}
.dv-empty-glyph {
  font: 38px var(--font-mono);
  opacity: 0.4;
}
.dv-layer-empty .davinci-ghost {
  font-size: 10px;
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
.dv-trace-heading .davinci-hint {
  margin-left: auto;
  font: 9px var(--font-mono);
}
.dv-trace-heading .davinci-span {
  padding: 2px 5px;
  font-size: 9px;
}
.dv-trace-heading .davinci-ghost {
  flex-shrink: 0;
  padding: 2px 5px;
  font-size: 9px;
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
.dv-trace-content .davinci-why {
  font-size: 10px;
  padding: 3px 6px;
}
.dv-rule-arrow {
  opacity: 0.5;
  padding: 0 3px;
}
.dv-trace-note {
  margin-top: 8px;
  font-size: 10px;
  color: var(--color-notice);
}
.dv-workbench .davinci-provenance {
  background: color-mix(in srgb, var(--dv-l1) 15%, transparent);
  outline-color: var(--dv-l1);
}
.dv-static-dot {
  color: var(--dv-l1);
}
.dv-dynamic-dot {
  color: var(--dv-l3);
}
.dv-dump-notice {
  margin: 4px 0 0;
  font-size: 9px;
  color: var(--text-muted);
  line-height: 1.5;
}
.dv-trace-links {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 3;
}
.dv-trace-links path {
  fill: none;
  stroke: currentColor;
  stroke-width: 3;
  stroke-linecap: round;
}
.dv-trace-links .dv-trace-outline {
  stroke: var(--bg-primary);
  stroke-width: 7;
}
.dv-trace-links circle {
  fill: currentColor;
  stroke: var(--bg-primary);
  stroke-width: 2;
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
@media (min-width: 1700px) {
  .dv-layer .davinci-folio,
  .dv-layer .davinci-output .davinci-output-code.code-highlight {
    font-size: 12px;
  }
  .dv-layer-header {
    padding: 16px;
  }
  .dv-layer-heading h3 {
    font-size: 16px;
  }
  .dv-layer-header p {
    font-size: 11px;
  }
  .dv-layer-facts {
    font-size: 10px;
  }
}
@media (max-width: 1050px) {
  .dv-toolbar {
    flex-wrap: wrap;
    padding: 12px 14px;
  }
  .dv-brand h2 {
    font-size: 17px;
  }
  .dv-run-hint {
    display: none;
  }
  .dv-trace-heading {
    flex-wrap: wrap;
  }
}
</style>
