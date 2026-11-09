<script setup lang="ts">
defineOptions({ name: "InspectorPlaygroundStyles02" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.inspector-graph-kind,
.inspector-graph-edge-kind,
.inspector-graph-issues {
  border: 1px solid var(--border-color);
  border-radius: 999px;
  padding: 2px 8px;
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 0.6875rem;
}

.inspector-graph-summary {
  & span {
    border: 1px solid var(--border-color);
    border-radius: 999px;
    padding: 2px 8px;
    background: var(--bg-secondary);
    color: var(--text-muted);
    font-size: 0.6875rem;
  }
}

.inspector-graph {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 8px;
}

.inspector-graph-node {
  min-width: 0;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 10px;
}

.inspector-graph-node.active {
  border-color: var(--accent-rust);
}

.inspector-graph-node.issues {
  border-color: color-mix(in srgb, var(--color-warning) 42%, var(--border-color));
}

.inspector-graph-node-main {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.inspector-graph-file {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-issues {
  border-color: color-mix(in srgb, var(--color-warning) 55%, transparent);
  color: var(--color-warning);
}

.inspector-graph-node-meta {
  display: flex;
  gap: 8px;
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 0.6875rem;
}

.inspector-graph-edges {
  margin-top: 10px;
  border-top: 1px solid var(--border-color);
  padding-top: 8px;
}

.inspector-graph-edge {
  display: grid;
  grid-template-columns: auto 20px minmax(0, 1fr);
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
}

.inspector-graph-edge {
  & + .inspector-graph-edge {
    margin-top: 5px;
  }
}

.inspector-graph-target {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-arrow {
  color: var(--text-muted);
  text-align: center;
}

.inspector-graph-diagnostics {
  display: flex;
  max-height: 220px;
  flex-direction: column;
  gap: 6px;
  overflow: auto;
}

.inspector-graph-diagnostic {
  display: grid;
  grid-template-columns: minmax(76px, auto) minmax(90px, 0.8fr) minmax(0, 1.4fr);
  gap: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 8px;
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.inspector-graph-diagnostic.error {
  border-color: color-mix(in srgb, var(--color-error) 45%, var(--border-color));
}

.inspector-graph-diagnostic.warning {
  border-color: color-mix(in srgb, var(--color-warning) 45%, var(--border-color));
}

.inspector-graph-diagnostic-code,
.inspector-graph-diagnostic-file {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-diagnostic-message {
  min-width: 0;
}

.inspector-url-note {
  margin-top: 8px;
  color: var(--color-warning);
  font-size: 0.75rem;
}

@media (max-width: 900px) {
  .inspector-diff-line {
    grid-template-columns: 38px 38px 20px minmax(0, 1fr);
  }

  .inspector-graph-diagnostic {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
