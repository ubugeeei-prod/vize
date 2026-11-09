<script setup lang="ts">
defineOptions({ name: "CroquisPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.export-badge.hoisted {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.export-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  margin-inline-start: 0.5rem;
  vertical-align: middle;
}

.badge.hoisted {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

/* Bindings Tab */
.source-group {
  margin-bottom: 1.25rem;
}

.source-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--border-primary);
}

.source-indicator {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.src-props {
  background: #a78bfa;
}
.src-emits {
  background: #f472b6;
}
.src-model {
  background: #fb923c;
}
.src-slots {
  background: #34d399;
}
.src-ref {
  background: #4ade80;
}
.src-reactive {
  background: #f87171;
}
.src-computed {
  background: #2dd4bf;
}
.src-import {
  background: #60a5fa;
}
.src-local {
  background: #94a3b8;
}
.src-function {
  background: #fbbf24;
}
.src-class {
  background: #818cf8;
}
.src-default {
  background: #6b7280;
}

.source-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.source-count {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-muted);
}

.binding-grid {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.binding-item {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  font-size: 0.75rem;
}

.binding-main {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.binding-name {
  font-weight: 600;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
}

.needs-value {
  font-size: 0.625rem;
  color: var(--color-success);
  opacity: 0.7;
}

.binding-meta {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--text-muted);
}

.binding-kind {
  font-family: "JetBrains Mono", monospace;
}

.binding-type {
  color: var(--color-info);
}

.binding-flags {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.flag {
  font-size: 0.5rem;
  text-transform: uppercase;
  padding: 0.0625rem 0.25rem;
  border-radius: 2px;
}

.flag.active {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.flag.inactive {
  background: var(--bg-tertiary);
  color: var(--text-muted);
  opacity: 0.5;
}

.flag.setup {
  background: var(--color-purple-bg);
  color: var(--color-purple);
}

.refs {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

/* Scopes Tab */
.scope-tree {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.scope-node {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.scope-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.scope-indicator {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  flex-shrink: 0;
}

.scope-kind {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.scope-range {
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.scope-bindings {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.375rem;
  padding-inline-start: 1.5rem;
}

.scope-binding {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-secondary);
}

/* Scope kind colors */
/* Template scopes */
.scope-module {
  background: rgba(167, 139, 250, 0.08);
}
.scope-module.scope-indicator {
  background: #a78bfa;
}

.scope-function {
  background: rgba(251, 191, 36, 0.08);
}
.scope-function.scope-indicator {
  background: #fbbf24;
}

.scope-block {
  background: rgba(148, 163, 184, 0.08);
}
.scope-block.scope-indicator {
  background: #94a3b8;
}

.scope-vfor {
  background: rgba(74, 222, 128, 0.08);
}
.scope-vfor.scope-indicator {
  background: #4ade80;
}

.scope-vslot {
  background: rgba(45, 212, 191, 0.08);
}
.scope-vslot.scope-indicator {
  background: #2dd4bf;
}

.scope-event-handler {
  background: rgba(244, 114, 182, 0.08);
}
.scope-event-handler.scope-indicator {
  background: #f472b6;
}

.scope-callback {
  background: rgba(251, 146, 60, 0.08);
}
.scope-callback.scope-indicator {
  background: #fb923c;
}

/* Script scopes */
.scope-script-setup {
  background: rgba(96, 165, 250, 0.08);
}
.scope-script-setup.scope-indicator {
  background: #60a5fa;
}

.scope-non-script-setup {
  background: rgba(129, 140, 248, 0.08);
}
.scope-non-script-setup.scope-indicator {
  background: #818cf8;
}

/* SSR scopes */
.scope-universal {
  background: rgba(34, 211, 238, 0.08);
}
.scope-universal.scope-indicator {
  background: #22d3ee;
}

.scope-client-only {
  background: rgba(248, 113, 113, 0.08);
}
.scope-client-only.scope-indicator {
  background: #f87171;
}

/* JS Global scopes (runtime-specific) */
.scope-js-global-universal {
  background: rgba(253, 224, 71, 0.08);
}
.scope-js-global-universal.scope-indicator {
  background: #fde047;
}

.scope-js-global-browser {
  background: rgba(251, 146, 60, 0.08);
}
.scope-js-global-browser.scope-indicator {
  background: #fb923c;
}

.scope-js-global-node {
  background: rgba(74, 222, 128, 0.08);
}
.scope-js-global-node.scope-indicator {
  background: #4ade80;
}

.scope-js-global-deno {
  background: rgba(96, 165, 250, 0.08);
}
.scope-js-global-deno.scope-indicator {
  background: #60a5fa;
}

.scope-js-global-bun {
  background: rgba(244, 114, 182, 0.08);
}
.scope-js-global-bun.scope-indicator {
  background: #f472b6;
}

/* Vue global */
.scope-vue-global {
  background: rgba(52, 211, 153, 0.08);
}
.scope-vue-global.scope-indicator {
  background: #34d399;
}

.scope-external-module {
  background: rgba(192, 132, 252, 0.08);
}
.scope-external-module.scope-indicator {
  background: #c084fc;
}

.scope-default {
  background: var(--bg-secondary);
}
.scope-default.scope-indicator {
  background: var(--text-muted);
}
</style>
